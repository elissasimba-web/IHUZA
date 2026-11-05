-- Additive testnet merchant profile reviews. Legacy name-only votes remain as history.
DO $$
BEGIN
    IF current_database() <> 'ihuza_web3_testnet' THEN
        RAISE EXCEPTION 'IHUZA+ Web3 migration requires database ihuza_web3_testnet';
    END IF;
END
$$;

CREATE TABLE merchant_profile_versions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id uuid NOT NULL REFERENCES merchant_applications(account_id) ON DELETE RESTRICT,
    version integer NOT NULL CHECK (version > 0),
    applicant_name text NOT NULL CHECK (char_length(applicant_name) BETWEEN 2 AND 100 AND applicant_name = btrim(applicant_name)),
    company_name text CHECK (company_name IS NULL OR (char_length(company_name) BETWEEN 2 AND 120 AND company_name = btrim(company_name))),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (account_id, version)
);

CREATE INDEX merchant_profile_versions_current_idx ON merchant_profile_versions (account_id, version DESC);

CREATE TABLE merchant_profile_votes (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id uuid NOT NULL REFERENCES merchant_profile_versions(id) ON DELETE RESTRICT,
    reviewer_address text NOT NULL CHECK (reviewer_address ~ '^G[A-Z2-7]{55}$'),
    reviewer_set_hash text NOT NULL CHECK (reviewer_set_hash ~ '^[0-9a-f]{64}$'),
    profile_hash text NOT NULL CHECK (profile_hash ~ '^[0-9a-f]{64}$'),
    nonce_hex text NOT NULL UNIQUE CHECK (nonce_hex ~ '^[0-9a-f]{64}$'),
    issued_at timestamptz NOT NULL,
    expires_at timestamptz NOT NULL,
    signature_base64 text NOT NULL CHECK (char_length(signature_base64) = 88),
    approved_at timestamptz NOT NULL DEFAULT now(),
    CHECK (expires_at > issued_at AND expires_at <= issued_at + interval '5 minutes'),
    CHECK (approved_at >= issued_at AND approved_at < expires_at),
    UNIQUE (profile_id, reviewer_address, reviewer_set_hash)
);

CREATE INDEX merchant_profile_votes_profile_idx ON merchant_profile_votes (profile_id, reviewer_set_hash);

GRANT SELECT, INSERT ON merchant_profile_versions, merchant_profile_votes TO ihuza_web3_app;
