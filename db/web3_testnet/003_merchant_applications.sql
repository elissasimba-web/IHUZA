-- Testnet merchant requests only. Requests never grant merchant checkout.
DO $$
BEGIN
    IF current_database() <> 'ihuza_web3_testnet' THEN
        RAISE EXCEPTION 'IHUZA+ Web3 migration requires database ihuza_web3_testnet';
    END IF;
END
$$;

CREATE TABLE merchant_applications (
    account_id uuid PRIMARY KEY REFERENCES accounts(id) ON DELETE RESTRICT,
    proposed_name text NOT NULL
        CHECK (char_length(proposed_name) BETWEEN 2 AND 80
            AND proposed_name = btrim(proposed_name)),
    submitted_at timestamptz NOT NULL DEFAULT now()
);

GRANT SELECT, INSERT ON merchant_applications TO ihuza_web3_app;
