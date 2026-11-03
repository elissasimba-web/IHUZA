ALTER TABLE invoices
    ADD COLUMN checkout_idempotency_key TEXT,
    ADD COLUMN checkout_request_hash TEXT,
    ADD CONSTRAINT invoices_checkout_key_length
        CHECK (checkout_idempotency_key IS NULL OR length(checkout_idempotency_key) BETWEEN 16 AND 128),
    ADD CONSTRAINT invoices_checkout_hash_length
        CHECK (checkout_request_hash IS NULL OR length(checkout_request_hash) = 64),
    ADD CONSTRAINT invoices_checkout_key_hash_together
        CHECK ((checkout_idempotency_key IS NULL) = (checkout_request_hash IS NULL));

CREATE UNIQUE INDEX idx_invoices_customer_checkout_key
    ON invoices(customer_id, checkout_idempotency_key)
    WHERE checkout_idempotency_key IS NOT NULL;
