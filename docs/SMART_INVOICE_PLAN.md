# IHUZA+ — Smart Invoice draft ya testnet

Iki cyiciro kibika invoice draft off-chain. Umukiriya ufite session ahitamo product IDs n'ingano; API yongera gusoma ibiciro bya merchant wemewe, ikabara total, igakora snapshot itahindurwa na runtime role. `RWF` ni ikimenyetso cy'igiciro muri catalog, si asset yo kwishyura. Draft si fagitire ya EBM, receipt cyangwa gihamya y'ubwishyu.

## Migration 007 yasuzumwe mbere yo kuyikoresha

- `smart_invoice_drafts`: ID, customer na merchant account IDs, izina ry'ubucuruzi n'opaque QR reference byafashwe icyo gihe, `request_hash`, `idempotency_key`, total ya decimal, currency `RWF`, `status = draft`, igihe. Unique `(customer_account_id, idempotency_key)` ibuza duplicate ya retry. Ntihindura tables cyangwa migrations za kera.
- `smart_invoice_lines`: invoice ID, line number, product ID, product name, quantity, unit price na line total byafashwe icyo gihe. FK kuri product na invoice, unique `(invoice_id, line_number)`. Nta update/delete kuri runtime role; insert na select gusa. Database guard yemeza `ihuza_web3_testnet`.
- API ikora invoice n'imirongo yayo muri transaction imwe. Idempotency key isubiwemo n'urutonde rutandukanye yangwa. Gusoma draft bikorwa na customer nyirayo gusa. Nta wallet signature ya payment, transaction hash, EBM cyangwa payment status byongerwa.

## Igenzura

Mbere yo gukoresha 007: genzura `git diff --check`, SQL, grants n'uko 001–006 zidahinduwe; `web3_migrate.ps1` igaragaze 007 pending. Nyuma: reba installed, test ya totals, product y'undi merchant, unauthorized access, duplicate retry, same key/different request, snapshots n'uko invoice iguma draft.
