-- IHUZA+ Web3 testnet only. Apply only to a NEW database named
-- ihuza_web3_testnet. The guard below rejects every other database.

DO $$
BEGIN
    IF current_database() <> 'ihuza_web3_testnet' THEN
        RAISE EXCEPTION 'IHUZA+ Web3 migration requires database ihuza_web3_testnet';
    END IF;
END
$$;

CREATE TABLE accounts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    account_kind text NOT NULL DEFAULT 'person'
        CHECK (account_kind IN ('person', 'merchant')),
    qr_reference text NOT NULL UNIQUE
        CHECK (qr_reference ~ '^[A-Za-z0-9_-]{43}$'),
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE wallet_links (
    account_id uuid PRIMARY KEY REFERENCES accounts(id) ON DELETE RESTRICT,
    network_passphrase text NOT NULL
        CHECK (network_passphrase = 'Test SDF Network ; September 2015'),
    stellar_address text NOT NULL
        CHECK (stellar_address ~ '^G[A-Z2-7]{55}$'),
    linked_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (network_passphrase, stellar_address)
);

CREATE TABLE wallet_challenges (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    network_passphrase text NOT NULL
        CHECK (network_passphrase = 'Test SDF Network ; September 2015'),
    stellar_address text NOT NULL
        CHECK (stellar_address ~ '^G[A-Z2-7]{55}$'),
    nonce_hex text NOT NULL UNIQUE
        CHECK (nonce_hex ~ '^[0-9a-f]{64}$'),
    issued_at timestamptz NOT NULL,
    expires_at timestamptz NOT NULL,
    consumed_at timestamptz,
    purpose text NOT NULL CHECK (purpose = 'authenticate'),
    CHECK (expires_at > issued_at AND expires_at <= issued_at + interval '5 minutes'),
    CHECK (consumed_at IS NULL OR consumed_at >= issued_at)
);

CREATE INDEX wallet_challenges_pending_idx
    ON wallet_challenges (stellar_address, expires_at)
    WHERE consumed_at IS NULL;

CREATE TABLE security_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id uuid REFERENCES accounts(id) ON DELETE RESTRICT,
    challenge_id uuid REFERENCES wallet_challenges(id) ON DELETE RESTRICT,
    event_kind text NOT NULL CHECK (event_kind IN (
        'challenge_issued', 'proof_accepted', 'proof_rejected', 'account_created'
    )),
    occurred_at timestamptz NOT NULL DEFAULT now()
);
