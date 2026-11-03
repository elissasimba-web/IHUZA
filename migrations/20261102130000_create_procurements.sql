ALTER TABLE suppliers
    ADD CONSTRAINT suppliers_id_merchant_unique UNIQUE (id, merchant_id);

ALTER TABLE products
    ADD CONSTRAINT products_id_merchant_unique UNIQUE (id, merchant_id);

CREATE TABLE procurements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL REFERENCES merchant_profiles(id) ON DELETE CASCADE,
    supplier_id UUID NOT NULL,
    supplier_name TEXT NOT NULL,
    procurement_number TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'DRAFT',
    payment_status TEXT NOT NULL DEFAULT 'UNPAID',
    currency CHAR(3) NOT NULL DEFAULT 'RWF',
    subtotal NUMERIC(20, 2) NOT NULL DEFAULT 0,
    total_amount NUMERIC(20, 2) NOT NULL DEFAULT 0,
    procured_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT procurements_supplier_belongs_to_merchant
        FOREIGN KEY (supplier_id, merchant_id)
        REFERENCES suppliers (id, merchant_id)
        ON DELETE RESTRICT,
    CONSTRAINT procurements_merchant_number_unique
        UNIQUE (merchant_id, procurement_number),
    CONSTRAINT procurements_id_merchant_unique
        UNIQUE (id, merchant_id),
    CONSTRAINT procurements_status_valid
        CHECK (status IN ('DRAFT', 'RECEIVED', 'CANCELLED')),
    CONSTRAINT procurements_payment_status_valid
        CHECK (payment_status IN ('UNPAID', 'PARTIALLY_PAID', 'PAID')),
    CONSTRAINT procurements_currency_not_empty
        CHECK (currency <> ''),
    CONSTRAINT procurements_amounts_non_negative
        CHECK (subtotal >= 0 AND total_amount >= 0)
);

CREATE INDEX idx_procurements_merchant_procured_at
    ON procurements (merchant_id, procured_at DESC);

CREATE INDEX idx_procurements_supplier_procured_at
    ON procurements (supplier_id, procured_at DESC);

CREATE TABLE procurement_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    procurement_id UUID NOT NULL,
    merchant_id UUID NOT NULL,
    product_id UUID NOT NULL,
    product_name TEXT NOT NULL,
    quantity NUMERIC(20, 3) NOT NULL,
    unit_cost NUMERIC(20, 2) NOT NULL,
    line_total NUMERIC(20, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT procurement_items_procurement_belongs_to_merchant
        FOREIGN KEY (procurement_id, merchant_id)
        REFERENCES procurements (id, merchant_id)
        ON DELETE CASCADE,
    CONSTRAINT procurement_items_product_belongs_to_merchant
        FOREIGN KEY (product_id, merchant_id)
        REFERENCES products (id, merchant_id)
        ON DELETE RESTRICT,
    CONSTRAINT procurement_items_quantity_positive
        CHECK (quantity > 0),
    CONSTRAINT procurement_items_unit_cost_non_negative
        CHECK (unit_cost >= 0),
    CONSTRAINT procurement_items_line_total_non_negative
        CHECK (line_total >= 0)
);

CREATE INDEX idx_procurement_items_procurement_id
    ON procurement_items (procurement_id);

CREATE INDEX idx_procurement_items_product_id
    ON procurement_items (product_id);
