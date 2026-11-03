 CREATE TABLE invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL REFERENCES merchant_profiles(id) ON DELETE RESTRICT,
    customer_id UUID REFERENCES users(id) ON DELETE SET NULL,
    invoice_number TEXT NOT NULL,
    subtotal NUMERIC(20, 2) NOT NULL,
    total_amount NUMERIC(20, 2) NOT NULL,
    currency CHAR(3) NOT NULL DEFAULT 'RWF',
    payment_status TEXT NOT NULL DEFAULT 'PENDING',
    invoice_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    qr_reference TEXT UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT invoices_subtotal_non_negative
        CHECK (subtotal >= 0),

    CONSTRAINT invoices_total_non_negative
        CHECK (total_amount >= 0),

    CONSTRAINT invoices_currency_not_empty
        CHECK (currency <> ''),

    CONSTRAINT invoices_payment_status_valid
        CHECK (payment_status IN ('PENDING', 'PAID', 'CANCELLED', 'REFUNDED'))
);

CREATE UNIQUE INDEX idx_invoices_merchant_number
    ON invoices(merchant_id, invoice_number);

CREATE INDEX idx_invoices_customer_id
    ON invoices(customer_id);

CREATE INDEX idx_invoices_merchant_date
    ON invoices(merchant_id, invoice_date);

CREATE INDEX idx_invoices_payment_status
    ON invoices(payment_status);


CREATE TABLE invoice_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    product_id UUID REFERENCES products(id) ON DELETE SET NULL,
    product_name TEXT NOT NULL,
    quantity NUMERIC(20, 3) NOT NULL,
    unit_price NUMERIC(20, 2) NOT NULL,
    subtotal NUMERIC(20, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT invoice_items_quantity_positive
        CHECK (quantity > 0),

    CONSTRAINT invoice_items_unit_price_non_negative
        CHECK (unit_price >= 0),

    CONSTRAINT invoice_items_subtotal_non_negative
        CHECK (subtotal >= 0)
);

CREATE INDEX idx_invoice_items_invoice_id
    ON invoice_items(invoice_id);

CREATE INDEX idx_invoice_items_product_id
    ON invoice_items(product_id);