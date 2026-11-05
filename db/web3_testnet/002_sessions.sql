-- IHUZA+ localhost Stellar testnet identity sessions only.
DO $$
BEGIN
    IF current_database() <> 'ihuza_web3_testnet' THEN
        RAISE EXCEPTION 'IHUZA+ Web3 migration requires database ihuza_web3_testnet';
    END IF;
END
$$;

CREATE TABLE account_sessions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id uuid NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    token_hash bytea NOT NULL UNIQUE CHECK (octet_length(token_hash) = 32),
    created_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL,
    revoked_at timestamptz,
    CHECK (expires_at > created_at AND expires_at <= created_at + interval '12 hours'),
    CHECK (revoked_at IS NULL OR revoked_at >= created_at)
);

CREATE INDEX account_sessions_account_idx ON account_sessions (account_id, expires_at)
    WHERE revoked_at IS NULL;

CREATE TABLE session_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id uuid NOT NULL REFERENCES account_sessions(id) ON DELETE RESTRICT,
    event_kind text NOT NULL CHECK (event_kind IN ('created', 'revoked')),
    occurred_at timestamptz NOT NULL DEFAULT now()
);

GRANT SELECT, INSERT, UPDATE (revoked_at) ON account_sessions TO ihuza_web3_app;
GRANT SELECT, INSERT ON session_events TO ihuza_web3_app;
