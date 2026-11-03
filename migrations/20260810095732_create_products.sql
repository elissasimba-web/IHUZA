 CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL REFERENCES merchant_profiles(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    price NUMERIC(20, 2) NOT NULL,
    currency CHAR(3) NOT NULL DEFAULT 'RWF',
    sku TEXT,
    barcode TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT products_price_non_negative
        CHECK (price >= 0),

    CONSTRAINT products_currency_not_empty
        CHECK (currency <> '')
);

CREATE INDEX idx_products_merchant_id
    ON products(merchant_id);

CREATE INDEX idx_products_merchant_active
    ON products(merchant_id, is_active);

CREATE UNIQUE INDEX idx_products_merchant_sku
    ON products(merchant_id, sku)
    WHERE sku IS NOT NULL;

CREATE UNIQUE INDEX idx_products_merchant_barcode
    ON products(merchant_id, barcode)
    WHERE barcode IS NOT NULL;


CREATE TABLE product_images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    image_url TEXT NOT NULL,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_product_images_product_id
    ON product_images(product_id);

CREATE UNIQUE INDEX idx_product_images_one_primary
    ON product_images(product_id)
    WHERE is_primary = TRUE;
