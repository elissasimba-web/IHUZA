-- Production wallet ledger foundation. No funding or transfer API is enabled by this migration.
-- Customer value must enter only after a separately verified and safeguarded funding flow exists.
CREATE TABLE wallet_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE RESTRICT,
    currency CHAR(3) NOT NULL DEFAULT 'RWF' CHECK (btrim(currency) = 'RWF'),
    status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'FROZEN')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE FUNCTION wallet_account_guard() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'wallet accounts cannot be deleted';
    END IF;
    IF NEW.id IS DISTINCT FROM OLD.id OR NEW.user_id IS DISTINCT FROM OLD.user_id
       OR NEW.currency IS DISTINCT FROM OLD.currency
       OR NEW.created_at IS DISTINCT FROM OLD.created_at THEN
        RAISE EXCEPTION 'wallet account identity is immutable';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER wallet_account_guard_trigger
BEFORE UPDATE OR DELETE ON wallet_accounts
FOR EACH ROW EXECUTE FUNCTION wallet_account_guard();

-- At this stage only internal transfers are modeled. Verified funding, withdrawals,
-- reversals, fees, and reserve accounts require separate reviewed migrations.
CREATE TABLE wallet_ledger_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sender_account_id UUID NOT NULL REFERENCES wallet_accounts(id) ON DELETE RESTRICT,
    recipient_account_id UUID NOT NULL REFERENCES wallet_accounts(id) ON DELETE RESTRICT,
    amount NUMERIC(20, 2) NOT NULL CHECK (amount > 0),
    currency CHAR(3) NOT NULL DEFAULT 'RWF' CHECK (btrim(currency) = 'RWF'),
    request_key TEXT NOT NULL CHECK (request_key ~ '^[A-Za-z0-9_-]{16,128}$'),
    request_hash CHAR(64) NOT NULL CHECK (btrim(request_hash) ~ '^[0-9a-f]{64}$'),
    status TEXT NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'POSTED')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    posted_at TIMESTAMPTZ,
    CONSTRAINT wallet_ledger_distinct_accounts CHECK (sender_account_id <> recipient_account_id),
    CONSTRAINT wallet_ledger_posted_time CHECK ((status = 'POSTED') = (posted_at IS NOT NULL)),
    CONSTRAINT wallet_ledger_sender_request_unique UNIQUE (sender_account_id, request_key)
);

CREATE INDEX idx_wallet_ledger_batches_recipient ON wallet_ledger_batches(recipient_account_id);
CREATE INDEX idx_wallet_ledger_batches_posted_at ON wallet_ledger_batches(posted_at) WHERE status = 'POSTED';

CREATE TABLE wallet_ledger_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_id UUID NOT NULL REFERENCES wallet_ledger_batches(id) ON DELETE RESTRICT,
    account_id UUID NOT NULL REFERENCES wallet_accounts(id) ON DELETE RESTRICT,
    amount NUMERIC(20, 2) NOT NULL CHECK (amount <> 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT wallet_ledger_one_entry_per_account UNIQUE (batch_id, account_id)
);

CREATE INDEX idx_wallet_ledger_entries_account ON wallet_ledger_entries(account_id);

CREATE FUNCTION wallet_ledger_entry_guard() RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE
    batch wallet_ledger_batches%ROWTYPE;
    account_currency CHAR(3);
BEGIN
    IF TG_OP <> 'INSERT' THEN
        RAISE EXCEPTION 'wallet ledger entries are append-only';
    END IF;
    SELECT * INTO batch FROM wallet_ledger_batches WHERE id = NEW.batch_id FOR UPDATE;
    IF NOT FOUND OR batch.status <> 'DRAFT' THEN
        RAISE EXCEPTION 'wallet entry requires a draft batch';
    END IF;
    IF NEW.account_id NOT IN (batch.sender_account_id, batch.recipient_account_id) THEN
        RAISE EXCEPTION 'wallet entry account does not belong to batch';
    END IF;
    SELECT currency INTO account_currency FROM wallet_accounts WHERE id = NEW.account_id;
    IF account_currency IS DISTINCT FROM batch.currency THEN
        RAISE EXCEPTION 'wallet entry currency mismatch';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER wallet_ledger_entry_guard_trigger
BEFORE INSERT OR UPDATE OR DELETE ON wallet_ledger_entries
FOR EACH ROW EXECUTE FUNCTION wallet_ledger_entry_guard();

CREATE FUNCTION wallet_ledger_batch_guard() RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE
    sender_status TEXT;
    recipient_status TEXT;
    sender_active BOOLEAN;
    recipient_active BOOLEAN;
    sender_balance NUMERIC;
    entry_count BIGINT;
    matching_entries BIGINT;
    net_amount NUMERIC;
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'wallet ledger batches cannot be deleted';
    END IF;
    IF OLD.status <> 'DRAFT' OR NEW.status <> 'POSTED'
       OR NEW.id IS DISTINCT FROM OLD.id
       OR NEW.sender_account_id IS DISTINCT FROM OLD.sender_account_id
       OR NEW.recipient_account_id IS DISTINCT FROM OLD.recipient_account_id
       OR NEW.amount IS DISTINCT FROM OLD.amount
       OR NEW.currency IS DISTINCT FROM OLD.currency
       OR NEW.request_key IS DISTINCT FROM OLD.request_key
       OR NEW.request_hash IS DISTINCT FROM OLD.request_hash
       OR NEW.created_at IS DISTINCT FROM OLD.created_at THEN
        RAISE EXCEPTION 'invalid wallet ledger batch transition';
    END IF;

    -- Account row locks serialize concurrent spends and are acquired in stable order.
    PERFORM 1 FROM wallet_accounts
    WHERE id IN (OLD.sender_account_id, OLD.recipient_account_id)
    ORDER BY id FOR UPDATE;
    SELECT a.status, u.is_active INTO sender_status, sender_active
    FROM wallet_accounts a JOIN users u ON u.id = a.user_id
    WHERE a.id = OLD.sender_account_id;
    SELECT a.status, u.is_active INTO recipient_status, recipient_active
    FROM wallet_accounts a JOIN users u ON u.id = a.user_id
    WHERE a.id = OLD.recipient_account_id;
    IF sender_status IS DISTINCT FROM 'ACTIVE' OR recipient_status IS DISTINCT FROM 'ACTIVE'
       OR sender_active IS DISTINCT FROM TRUE OR recipient_active IS DISTINCT FROM TRUE THEN
        RAISE EXCEPTION 'wallet account is not active';
    END IF;

    SELECT COUNT(*),
           COUNT(*) FILTER (WHERE (account_id = OLD.sender_account_id AND amount = -OLD.amount)
                                OR (account_id = OLD.recipient_account_id AND amount = OLD.amount)),
           COALESCE(SUM(amount), 0)
    INTO entry_count, matching_entries, net_amount
    FROM wallet_ledger_entries WHERE batch_id = OLD.id;
    IF entry_count <> 2 OR matching_entries <> 2 OR net_amount <> 0 THEN
        RAISE EXCEPTION 'wallet ledger batch must contain balanced sender and recipient entries';
    END IF;

    SELECT COALESCE(SUM(e.amount), 0) INTO sender_balance
    FROM wallet_ledger_entries e
    JOIN wallet_ledger_batches b ON b.id = e.batch_id AND b.status = 'POSTED'
    WHERE e.account_id = OLD.sender_account_id;
    IF sender_balance < OLD.amount THEN
        RAISE EXCEPTION 'insufficient wallet funds';
    END IF;
    NEW.posted_at := NOW();
    RETURN NEW;
END;
$$;

CREATE TRIGGER wallet_ledger_batch_guard_trigger
BEFORE UPDATE OR DELETE ON wallet_ledger_batches
FOR EACH ROW EXECUTE FUNCTION wallet_ledger_batch_guard();

CREATE VIEW wallet_balances AS
SELECT a.id AS account_id, a.user_id, btrim(a.currency) AS currency,
       COALESCE(SUM(e.amount) FILTER (WHERE b.status = 'POSTED'), 0)::NUMERIC(20, 2) AS balance
FROM wallet_accounts a
LEFT JOIN wallet_ledger_entries e ON e.account_id = a.id
LEFT JOIN wallet_ledger_batches b ON b.id = e.batch_id
GROUP BY a.id;
