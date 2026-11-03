 CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    invoice_id UUID REFERENCES invoices(id) ON DELETE SET NULL,

    sender_id UUID REFERENCES users(id) ON DELETE SET NULL,
    receiver_id UUID REFERENCES users(id) ON DELETE SET NULL,

    amount NUMERIC(20, 2) NOT NULL,
    currency CHAR(3) NOT NULL DEFAULT 'RWF',

    payment_method TEXT NOT NULL,
    payment_reference TEXT UNIQUE,
    status TEXT NOT NULL DEFAULT 'PENDING',

    transaction_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    receipt_reference TEXT UNIQUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT transactions_amount_positive
        CHECK (amount > 0),

    CONSTRAINT transactions_currency_not_empty
        CHECK (currency <> ''),

    CONSTRAINT transactions_status_valid
        CHECK (
            status IN (
                'PENDING',
                'PROCESSING',
                'SUCCESS',
                'FAILED',
                'CANCELLED',
                'REFUNDED'
            )
        )
);

CREATE INDEX idx_transactions_invoice_id
    ON transactions(invoice_id);

CREATE INDEX idx_transactions_sender_id
    ON transactions(sender_id);

CREATE INDEX idx_transactions_receiver_id
    ON transactions(receiver_id);

CREATE INDEX idx_transactions_status
    ON transactions(status);

CREATE INDEX idx_transactions_date
    ON transactions(transaction_date);

CREATE INDEX idx_transactions_payment_method
    ON transactions(payment_method);