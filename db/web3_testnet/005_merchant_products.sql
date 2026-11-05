-- Testnet off-chain catalog. Prices are display-only RWF amounts, not payment assets.
DO $$
BEGIN
    IF current_database() <> 'ihuza_web3_testnet' THEN
        RAISE EXCEPTION 'IHUZA+ Web3 migration requires database ihuza_web3_testnet';
    END IF;
END
$$;

CREATE TABLE merchant_products (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_account_id uuid NOT NULL REFERENCES merchant_applications(account_id) ON DELETE RESTRICT,
    name text NOT NULL CHECK (char_length(name) BETWEEN 2 AND 120 AND name = btrim(name)),
    unit_price numeric(20, 2) NOT NULL CHECK (unit_price > 0),
    currency_code text NOT NULL DEFAULT 'RWF' CHECK (currency_code = 'RWF'),
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX merchant_products_merchant_idx ON merchant_products (merchant_account_id, created_at, id);

GRANT SELECT, INSERT ON merchant_products TO ihuza_web3_app;
