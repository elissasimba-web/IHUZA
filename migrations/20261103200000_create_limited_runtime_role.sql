-- Schema migrations use the owner connection. The HTTP server uses a separate role.
-- This role cannot own or write wallet ledger history.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'ihuza_app') THEN
        RAISE EXCEPTION 'ihuza_app already exists; inspect its privileges before this migration';
    END IF;
    CREATE ROLE ihuza_app NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
    EXECUTE format('GRANT CONNECT ON DATABASE %I TO ihuza_app', current_database());
END;
$$;

GRANT USAGE ON SCHEMA public TO ihuza_app;

GRANT SELECT ON TABLE
    users, roles, user_roles, user_sessions, merchant_profiles,
    products, product_images, invoices, invoice_items,
    login_rate_limits, wallet_accounts, wallet_balances
TO ihuza_app;

GRANT INSERT ON TABLE
    users, user_roles, user_sessions, merchant_profiles,
    products, invoices, invoice_items,
    security_events, login_rate_limits, wallet_accounts
TO ihuza_app;

GRANT UPDATE ON TABLE
    users, user_sessions, merchant_profiles, products, login_rate_limits
TO ihuza_app;

GRANT DELETE ON TABLE login_rate_limits TO ihuza_app;

-- The test-only transfer function should not be callable by arbitrary roles.
REVOKE EXECUTE ON FUNCTION test_ledger_transfer(TEXT, TEXT, NUMERIC, TEXT) FROM PUBLIC;
