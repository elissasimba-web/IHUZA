-- A draft records the customer's request; it cannot move or reserve money.
CREATE TABLE wallet_transfer_intents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sender_user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    recipient_user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    amount NUMERIC(20, 2) NOT NULL CHECK (amount > 0),
    currency CHAR(3) NOT NULL DEFAULT 'RWF' CHECK (btrim(currency) = 'RWF'),
    request_key TEXT NOT NULL CHECK (request_key ~ '^[A-Za-z0-9_-]{16,128}$'),
    request_hash CHAR(64) NOT NULL CHECK (btrim(request_hash) ~ '^[0-9a-f]{64}$'),
    status TEXT NOT NULL DEFAULT 'DRAFT' CHECK (status = 'DRAFT'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT wallet_transfer_distinct_users CHECK (sender_user_id <> recipient_user_id),
    CONSTRAINT wallet_transfer_sender_key_unique UNIQUE (sender_user_id, request_key)
);

CREATE INDEX idx_wallet_transfer_intents_recipient ON wallet_transfer_intents(recipient_user_id);

-- The HTTP role can create and read drafts, but cannot change their state.
GRANT SELECT, INSERT ON wallet_transfer_intents TO ihuza_app;
