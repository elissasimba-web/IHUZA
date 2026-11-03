ALTER TABLE security_events
    DROP CONSTRAINT IF EXISTS security_events_event_type_check;

ALTER TABLE security_events
    ADD CONSTRAINT security_events_event_type_valid
    CHECK (
        event_type IN (
            'LOGIN_SUCCEEDED',
            'LOGIN_FAILED',
            'LOGIN_RATE_LIMITED',
            'LOGOUT_SUCCEEDED',
            'SESSION_REJECTED',
            'PASSWORD_CHANGED'
        )
    );
