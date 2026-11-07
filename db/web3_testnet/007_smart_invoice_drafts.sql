-- Immutable application-role snapshots for unpaid testnet invoice drafts.
DO $$
BEGIN
    IF current_database() <> 'ihuza_web3_testnet' THEN
        RAISE EXCEPTION 'IHUZA+ Web3 migration requires database ihuza_web3_testnet';
    END IF;
END
$$;

CREATE TABLE smart_invoice_drafts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_account_id uuid NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    merchant_account_id uuid NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    merchant_name text NOT NULL CHECK (char_length(merchant_name) BETWEEN 2 AND 80),
    merchant_reference text NOT NULL CHECK (char_length(merchant_reference) = 43),
    idempotency_key text NOT NULL CHECK (idempotency_key ~ '^[0-9a-f]{64}$'),
    request_hash text NOT NULL CHECK (request_hash ~ '^[0-9a-f]{64}$'),
    total numeric(24, 2) NOT NULL CHECK (total > 0),
    currency_code text NOT NULL DEFAULT 'RWF' CHECK (currency_code = 'RWF'),
    status text NOT NULL DEFAULT 'draft' CHECK (status = 'draft'),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (customer_account_id, idempotency_key)
);

CREATE INDEX smart_invoice_drafts_merchant_idx ON smart_invoice_drafts (merchant_account_id, created_at);

CREATE TABLE smart_invoice_lines (
    invoice_id uuid NOT NULL REFERENCES smart_invoice_drafts(id) ON DELETE RESTRICT,
    line_number integer NOT NULL CHECK (line_number BETWEEN 1 AND 20),
    product_id uuid NOT NULL REFERENCES merchant_products(id) ON DELETE RESTRICT,
    product_name text NOT NULL CHECK (char_length(product_name) BETWEEN 2 AND 120),
    quantity integer NOT NULL CHECK (quantity BETWEEN 1 AND 100),
    unit_price numeric(20, 2) NOT NULL CHECK (unit_price > 0),
    line_total numeric(22, 2) NOT NULL CHECK (line_total = unit_price * quantity),
    PRIMARY KEY (invoice_id, line_number),
    UNIQUE (invoice_id, product_id)
);

GRANT SELECT, INSERT ON smart_invoice_drafts, smart_invoice_lines TO ihuza_web3_app;
