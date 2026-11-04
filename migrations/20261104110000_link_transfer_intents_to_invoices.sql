-- Existing person drafts remain PERSON. Merchant invoice drafts use the same
-- intent table without introducing another QR or permitting money movement.
ALTER TABLE wallet_transfer_intents
    ADD COLUMN intent_kind TEXT NOT NULL DEFAULT 'PERSON',
    ADD COLUMN invoice_id UUID REFERENCES invoices(id) ON DELETE RESTRICT,
    ADD CONSTRAINT wallet_transfer_intent_kind_valid CHECK (
        (intent_kind = 'PERSON' AND invoice_id IS NULL)
        OR (intent_kind = 'MERCHANT_INVOICE' AND invoice_id IS NOT NULL)
    );

-- One draft per invoice avoids competing payment preparations at this stage.
CREATE UNIQUE INDEX idx_wallet_transfer_intents_invoice_unique
    ON wallet_transfer_intents(invoice_id) WHERE invoice_id IS NOT NULL;
