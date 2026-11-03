CREATE TABLE security_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    event_type TEXT NOT NULL,
    identifier_hash TEXT,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,

    CHECK (
        event_type IN (
            'LOGIN_SUCCEEDED',
            'LOGIN_FAILED',
            'LOGIN_RATE_LIMITED',
            'LOGOUT_SUCCEEDED',
            'SESSION_REJECTED'
        )
    ),
    CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX idx_security_events_user_occurred_at
    ON security_events (user_id, occurred_at DESC);

CREATE INDEX idx_security_events_identifier_occurred_at
    ON security_events (identifier_hash, occurred_at DESC)
    WHERE identifier_hash IS NOT NULL;

CREATE INDEX idx_security_events_type_occurred_at
    ON security_events (event_type, occurred_at DESC);

CREATE TABLE login_rate_limits (
    identifier_hash TEXT PRIMARY KEY,
    failed_attempts INTEGER NOT NULL DEFAULT 0,
    window_started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    locked_until TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CHECK (failed_attempts >= 0),
    CHECK (locked_until IS NULL OR locked_until >= window_started_at)
);
