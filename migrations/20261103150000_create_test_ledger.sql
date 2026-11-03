-- Isolated test-only ledger. No payment, invoice, or receipt table references it.
CREATE TABLE test_ledger_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_ref TEXT NOT NULL UNIQUE,
    kind TEXT NOT NULL CHECK (kind IN ('USER', 'TEST_SOURCE')),
    currency CHAR(3) NOT NULL DEFAULT 'RWF' CHECK (btrim(currency) = 'RWF'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT test_ledger_account_ref_valid CHECK (account_ref ~ '^[A-Za-z0-9_-]{16,128}$')
);

CREATE TABLE test_ledger_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_key TEXT NOT NULL UNIQUE,
    request_hash CHAR(64) NOT NULL CHECK (btrim(request_hash) ~ '^[0-9a-f]{64}$'),
    status TEXT NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'POSTED')),
    currency CHAR(3) NOT NULL DEFAULT 'RWF' CHECK (btrim(currency) = 'RWF'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    posted_at TIMESTAMPTZ,
    CONSTRAINT test_ledger_key_valid CHECK (request_key ~ '^[A-Za-z0-9_-]{16,128}$'),
    CONSTRAINT test_ledger_posted_time_valid CHECK ((status = 'POSTED') = (posted_at IS NOT NULL))
);

CREATE TABLE test_ledger_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_id UUID NOT NULL REFERENCES test_ledger_batches(id) ON DELETE RESTRICT,
    account_id UUID NOT NULL REFERENCES test_ledger_accounts(id) ON DELETE RESTRICT,
    amount NUMERIC(20, 2) NOT NULL CHECK (amount <> 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_test_ledger_entries_batch ON test_ledger_entries(batch_id);
CREATE INDEX idx_test_ledger_entries_account ON test_ledger_entries(account_id);

CREATE FUNCTION test_ledger_forbid_account_change() RETURNS TRIGGER
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'test ledger accounts are immutable';
END;
$$;

CREATE TRIGGER test_ledger_account_immutable
BEFORE UPDATE OR DELETE ON test_ledger_accounts
FOR EACH ROW EXECUTE FUNCTION test_ledger_forbid_account_change();

CREATE FUNCTION test_ledger_guard_entry() RETURNS TRIGGER
LANGUAGE plpgsql AS $$
DECLARE
    batch_status TEXT;
    account_currency CHAR(3);
    batch_currency CHAR(3);
BEGIN
    IF TG_OP <> 'INSERT' THEN
        RAISE EXCEPTION 'test ledger entries are append-only';
    END IF;
    SELECT status, currency INTO batch_status, batch_currency
    FROM test_ledger_batches WHERE id = NEW.batch_id FOR UPDATE;
    IF batch_status IS DISTINCT FROM 'DRAFT' THEN
        RAISE EXCEPTION 'entries require a draft test ledger batch';
    END IF;
    SELECT currency INTO account_currency
    FROM test_ledger_accounts WHERE id = NEW.account_id;
    IF account_currency IS DISTINCT FROM batch_currency THEN
        RAISE EXCEPTION 'test ledger account and batch currencies differ';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER test_ledger_entry_guard
BEFORE INSERT OR UPDATE OR DELETE ON test_ledger_entries
FOR EACH ROW EXECUTE FUNCTION test_ledger_guard_entry();

CREATE FUNCTION test_ledger_guard_batch() RETURNS TRIGGER
LANGUAGE plpgsql AS $$
DECLARE
    entry_count BIGINT;
    distinct_accounts BIGINT;
    net_amount NUMERIC;
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'test ledger batches cannot be deleted';
    END IF;
    IF OLD.status <> 'DRAFT' OR NEW.status <> 'POSTED'
       OR NEW.id IS DISTINCT FROM OLD.id
       OR NEW.request_key IS DISTINCT FROM OLD.request_key
       OR NEW.request_hash IS DISTINCT FROM OLD.request_hash
       OR NEW.currency IS DISTINCT FROM OLD.currency THEN
        RAISE EXCEPTION 'invalid test ledger batch transition';
    END IF;
    SELECT COUNT(*), COUNT(DISTINCT account_id), COALESCE(SUM(amount), 0)
    INTO entry_count, distinct_accounts, net_amount
    FROM test_ledger_entries WHERE batch_id = OLD.id;
    IF entry_count < 2 OR distinct_accounts < 2 OR net_amount <> 0 THEN
        RAISE EXCEPTION 'test ledger batch must have balanced entries on two accounts';
    END IF;
    NEW.posted_at := NOW();
    RETURN NEW;
END;
$$;

CREATE TRIGGER test_ledger_batch_guard
BEFORE UPDATE OR DELETE ON test_ledger_batches
FOR EACH ROW EXECUTE FUNCTION test_ledger_guard_batch();

CREATE VIEW test_ledger_balances AS
SELECT e.account_id, btrim(a.currency) AS currency, SUM(e.amount) AS balance
FROM test_ledger_entries e
JOIN test_ledger_batches b ON b.id = e.batch_id AND b.status = 'POSTED'
JOIN test_ledger_accounts a ON a.id = e.account_id
GROUP BY e.account_id, a.currency;

CREATE FUNCTION test_ledger_transfer(
    from_ref TEXT, to_ref TEXT, transfer_amount NUMERIC, idempotency_key TEXT
) RETURNS UUID
LANGUAGE plpgsql AS $$
DECLARE
    from_id UUID;
    to_id UUID;
    from_kind TEXT;
    current_balance NUMERIC;
    existing_id UUID;
    existing_hash CHAR(64);
    payload_hash CHAR(64);
    new_id UUID;
BEGIN
    IF from_ref IS NULL OR to_ref IS NULL OR idempotency_key IS NULL
       OR from_ref !~ '^[A-Za-z0-9_-]{16,128}$'
       OR to_ref !~ '^[A-Za-z0-9_-]{16,128}$'
       OR from_ref = to_ref OR transfer_amount <= 0 OR transfer_amount IS NULL
       OR scale(transfer_amount) > 2 OR transfer_amount > 999999999999999999.99
       OR idempotency_key !~ '^[A-Za-z0-9_-]{16,128}$' THEN
        RAISE EXCEPTION 'invalid test ledger transfer';
    END IF;
    payload_hash := encode(digest(
        from_ref || '|' || to_ref || '|' || (transfer_amount::NUMERIC(20,2))::TEXT,
        'sha256'), 'hex');
    PERFORM pg_advisory_xact_lock(hashtextextended('test-ledger:' || idempotency_key, 0));
    SELECT id, request_hash INTO existing_id, existing_hash
    FROM test_ledger_batches WHERE request_key = idempotency_key;
    IF FOUND THEN
        IF existing_hash <> payload_hash THEN
            RAISE EXCEPTION 'test ledger idempotency key reused with different transfer';
        END IF;
        RETURN existing_id;
    END IF;

    -- Lock both account rows in one order so concurrent opposite transfers serialize.
    PERFORM 1 FROM test_ledger_accounts
    WHERE account_ref IN (from_ref, to_ref) ORDER BY id FOR UPDATE;
    SELECT id, kind INTO from_id, from_kind
    FROM test_ledger_accounts WHERE account_ref = from_ref;
    SELECT id INTO to_id
    FROM test_ledger_accounts WHERE account_ref = to_ref;
    IF from_id IS NULL OR to_id IS NULL THEN
        RAISE EXCEPTION 'test ledger account not found';
    END IF;
    IF from_kind = 'USER' THEN
        SELECT COALESCE(balance, 0) INTO current_balance
        FROM test_ledger_balances WHERE account_id = from_id;
        IF COALESCE(current_balance, 0) < transfer_amount THEN
            RAISE EXCEPTION 'insufficient test ledger funds';
        END IF;
    END IF;

    INSERT INTO test_ledger_batches (request_key, request_hash)
    VALUES (idempotency_key, payload_hash) RETURNING id INTO new_id;
    INSERT INTO test_ledger_entries (batch_id, account_id, amount)
    VALUES (new_id, from_id, -transfer_amount),
           (new_id, to_id, transfer_amount);
    UPDATE test_ledger_batches SET status = 'POSTED' WHERE id = new_id;
    RETURN new_id;
END;
$$;
