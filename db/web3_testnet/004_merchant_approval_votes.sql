-- Testnet 2-of-3 merchant review attestations. No payment authority.
DO $$
BEGIN
    IF current_database() <> 'ihuza_web3_testnet' THEN
        RAISE EXCEPTION 'IHUZA+ Web3 migration requires database ihuza_web3_testnet';
    END IF;
END
$$;

CREATE TABLE merchant_approval_votes (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id uuid NOT NULL REFERENCES merchant_applications(account_id) ON DELETE RESTRICT,
    reviewer_address text NOT NULL CHECK (reviewer_address ~ '^G[A-Z2-7]{55}$'),
    reviewer_set_hash text NOT NULL CHECK (reviewer_set_hash ~ '^[0-9a-f]{64}$'),
    name_hash text NOT NULL CHECK (name_hash ~ '^[0-9a-f]{64}$'),
    nonce_hex text NOT NULL UNIQUE CHECK (nonce_hex ~ '^[0-9a-f]{64}$'),
    issued_at timestamptz NOT NULL,
    expires_at timestamptz NOT NULL,
    signature_base64 text NOT NULL CHECK (char_length(signature_base64) = 88),
    approved_at timestamptz NOT NULL DEFAULT now(),
    CHECK (expires_at > issued_at AND expires_at <= issued_at + interval '5 minutes'),
    CHECK (approved_at >= issued_at AND approved_at < expires_at),
    UNIQUE (account_id, reviewer_address, reviewer_set_hash)
);

CREATE INDEX merchant_approval_votes_account_idx ON merchant_approval_votes (account_id, reviewer_set_hash);

GRANT SELECT, INSERT ON merchant_approval_votes TO ihuza_web3_app;
