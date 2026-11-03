ALTER TABLE users
    ADD COLUMN account_qr_reference TEXT;

UPDATE users
SET account_qr_reference = encode(gen_random_bytes(24), 'hex');

ALTER TABLE users
    ALTER COLUMN account_qr_reference SET DEFAULT encode(gen_random_bytes(24), 'hex'),
    ALTER COLUMN account_qr_reference SET NOT NULL;

ALTER TABLE users
    ADD CONSTRAINT users_account_qr_reference_unique UNIQUE (account_qr_reference);
