 
CREATE TABLE suppliers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL REFERENCES merchant_profiles(id) ON DELETE CASCADE,

    name TEXT NOT NULL,
    phone_number TEXT,
    email TEXT,
    address TEXT,
    tax_identification_number TEXT,

    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_suppliers_merchant_id
    ON suppliers(merchant_id);

CREATE INDEX idx_suppliers_merchant_active
    ON suppliers(merchant_id, is_active);

CREATE UNIQUE INDEX idx_suppliers_merchant_tax_id
    ON suppliers(merchant_id, tax_identification_number)
    WHERE tax_identification_number IS NOT NULL;