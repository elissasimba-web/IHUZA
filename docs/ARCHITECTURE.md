# IHUZA+ — Stellar Web3 architecture

## Aho tugeze

Iri shami rifite Soroban `intent_anchor` contract, read-only testnet probe, frontend ihuza Freighter wallet kuri testnet ikayisaba gusinya challenge umukoresha amaze gusoma, Rust library igenzura SEP-53 signature, database ya testnet ifite identity na session tables, na API ya localhost itanga kandi ikemeza wallet challenges. Iyo proof yemejwe, API ikora konti ifite QR reference ihoraho cyangwa igasoma iyariho, igatanga session y'amasaha 12. Logout iyifunga. Nta QR resolver, chain indexer, payment cyangwa Receipt Vault birimo. Mu iyimurika bigomba kuvugwa uko biri.

PostgreSQL ya `ihuza_web3_testnet` ibika identity metadata na challenges. API iyikoresha ifite database user ifite uburenganzira buke; Vite dev proxy ihuza frontend na API kuri localhost. Ubutumwa businywa burimo origin ya local development, purpose, network, address, nonce n'igihe; frontend igenzura ibyo na signer mbere yo kohereza proof. SEP-53 signature yemeza urufunguzo rwasinye, ariko si uburenganzira bwose bwa Stellar multisig account cyangwa payment proof.

## Aho buri kintu kizaba

| Stellar / Soroban | IHUZA+ API na database | Client y'umukoresha |
| --- | --- | --- |
| Nyiri asset, uburenganzira bw'isinywa, transaction result n'events zikenewe | Identity, role checks, QR reference resolver, merchant catalog, Smart Invoice snapshots, receipts, notifications na audit | QR scan, catalog, invoice review, wallet connection, gusinya no kwerekana status yagenzuwe |

PostgreSQL migrations aracyakenewe ku makuru ya off-chain. Gukoresha database ntibibuza app kuba Web3; icyo tugenzura ni uko server idashobora gukoresha amafaranga y'umukiriya itabisinyiwe kandi ko payment status iva mu bimenyetso bya chain byagenzuwe. Soroban contract ikoreshwa gusa aho amategeko y'ubwishyu asaba programmable logic; native Stellar payment ntigomba gusubirwamo nta mpamvu.

## Umutekano n'uburyo bwo kubaka

1. Hitamo wallet UX na recovery model idaha server private keys. Huze konti na Stellar address ukoresheje challenge ifite network, expiry na signature yagenzuwe. Server igomba gutanga nonce itabasha gukekwa, kuyibika ku konti iri mu session, no kuyikoresha rimwe gusa mu gikorwa kimwe cya database transaction. Signature yonyine ntihagije mu guhuza konti; Stellar multisig authorization isaba igenzura ryihariye.
2. Wubake identity, merchant approval, catalog n'account QR imwe mu API nshya. Ibirimo QR bibe opaque kandi bidashobora gukekwa.
3. Wubake quote na Smart Invoice ibarwa na server, ifite snapshots z'ibiciro n'idempotency. Payer arebe network, asset, recipient, amount na expiry mbere yo gusinya.
4. Ohereza cyangwa urebe transaction; genzura network, source, destination, asset, amount, success na unique reference byigenga. Indexing na receipt creation bibe idempotent kandi bishobore gusubirwamo nyuma y'ikosa.
5. Huza Receipt Vault, notifications na Merchant Assistant n'ubwishyu bwemejwe. Reconciliation, refunds, disputes, rate limits, audit n'ibizamini bya concurrency bikorwe mbere y'amafaranga nyayo.

`intent_anchor` iriho ubu yerekana gusa `require_auth`, expiry na replay guard kuri hash. Hash encoding nyayo, storage lifetime, wallet challenge, privacy review na external audit ntibirakorwa; iyo contract ntigomba gukoreshwa mu kwishyura uko iri.
