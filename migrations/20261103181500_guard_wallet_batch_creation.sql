-- A batch must start as DRAFT. Only the checked DRAFT -> POSTED update may post it.
CREATE FUNCTION wallet_ledger_batch_insert_guard() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.status <> 'DRAFT' OR NEW.posted_at IS NOT NULL THEN
        RAISE EXCEPTION 'wallet ledger batches must be created as drafts';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER wallet_ledger_batch_insert_guard_trigger
BEFORE INSERT ON wallet_ledger_batches
FOR EACH ROW EXECUTE FUNCTION wallet_ledger_batch_insert_guard();
