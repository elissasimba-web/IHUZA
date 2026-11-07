# IHUZA+ — Stellar Web3 architecture

## Aho tugeze

Merchant approval ya testnet ubu isinyira profile version irimo izina ry'uwasabye, iry'ubucuruzi n'irya company iyo rihari. Votes za kera zasinyiraga izina rimwe gusa ziguma mu mateka ariko ntizongera gufungura QR cyangwa catalog. Iyi approval si KYC, EBM cyangwa uburenganzira bwa delivery.

Quote preview ibarwa na API ku biciro biri muri database n'ingano zatoranyijwe n'umukiriya winjiye. API ishobora kubika Smart Invoice draft ifite snapshots n'idempotency. Nta payment, chain verification cyangwa receipt bihari muri iyi ntambwe; draft ntivuga ko yishyuwe.

Payment confirmation izasaba kugenzura transaction yatsinze muri ledger n'operation ihuye na source, destination, asset, amount na reference. Uwayohereje n'uwakiriye bazabona status imwe nyuma y'iryo genzura; acknowledgement y'uwakiriye izaba iy'itumanaho gusa. Reba `docs/PAYMENT_CONFIRMATION_PLAN.md`.

Iri shami rifite Soroban `intent_anchor` contract, read-only testnet probe, frontend ihuza Freighter wallet kuri testnet ikayisaba gusinya challenge umukoresha amaze gusoma, Rust library igenzura SEP-53 signature, database ya testnet ifite identity, sessions, merchant applications, approval votes na catalog, na API ya localhost itanga kandi ikemeza wallet challenges. Iyo proof yemejwe, API ikora konti ifite QR reference ihoraho cyangwa igasoma iyariho, igatanga session y'amasaha 12. Logout iyifunga. Browser ishushanya QR irimo opaque reference gusa; API isobanura reference nta konti ID, wallet address cyangwa amakuru y'ibanga itangaje. Nyiri session ashobora gusaba kuba umucuruzi. Reviewer addresses eshatu zashyizwe muri config zishobora gutanga imikono ya SEP-53; votes ebyiri zemewe zitanga merchant status kuri testnet. Merchant wemejwe ashobora kongeramo ibicuruzwa n'ibiciro; catalog ibonwa hakoreshejwe QR reference imwe. Iyo config ibuze, review na catalog birafungwa. Checkout ikomeza gufungwa n'iyo merchant yemejwe. Nta camera scanner, chain indexer, payment cyangwa Receipt Vault birimo. Mu iyimurika bigomba kuvugwa uko biri.

PostgreSQL ya `ihuza_web3_testnet` ibika identity metadata na challenges. API iyikoresha ifite database user ifite uburenganzira buke; Vite dev proxy ihuza frontend na API kuri localhost. Ubutumwa businywa burimo origin ya local development, purpose, network, address, nonce n'igihe; frontend igenzura ibyo na signer mbere yo kohereza proof. SEP-53 signature yemeza urufunguzo rwasinye, ariko si uburenganzira bwose bwa Stellar multisig account cyangwa payment proof.

## Aho buri kintu kizaba

| Stellar / Soroban | IHUZA+ API na database | Client y'umukoresha |
| --- | --- | --- |
| Nyiri asset, uburenganzira bw'isinywa, transaction result n'events zikenewe | Identity, role checks, QR reference resolver, merchant catalog, Smart Invoice snapshots, receipts, notifications na audit | QR scan, catalog, invoice review, wallet connection, gusinya no kwerekana status yagenzuwe |

PostgreSQL migrations aracyakenewe ku makuru ya off-chain. Gukoresha database ntibibuza app kuba Web3; icyo tugenzura ni uko server idashobora gukoresha amafaranga y'umukiriya itabisinyiwe kandi ko payment status iva mu bimenyetso bya chain byagenzuwe. Soroban contract ikoreshwa gusa aho amategeko y'ubwishyu asaba programmable logic; native Stellar payment ntigomba gusubirwamo nta mpamvu.

## Umutekano n'uburyo bwo kubaka

1. Hitamo wallet UX na recovery model idaha server private keys. Huze konti na Stellar address ukoresheje challenge ifite network, expiry na signature yagenzuwe. Server igomba gutanga nonce itabasha gukekwa, kuyibika ku konti iri mu session, no kuyikoresha rimwe gusa mu gikorwa kimwe cya database transaction. Signature yonyine ntihagije mu guhuza konti; Stellar multisig authorization isaba igenzura ryihariye.
2. Identity, QR imwe, approval ya testnet ya 2-of-3 na catalog y'ibanze biriho. Ibikurikira ni amabwiriza yo kugenzura ibimenyetso by'ubucuruzi, rejection/suspension, guhindura reviewer set n'ibiciro bifite amateka. Ibirimo QR bikomeza kuba opaque kandi bidashobora gukekwa.
3. Wubake quote na Smart Invoice ibarwa na server, ifite snapshots z'ibiciro n'idempotency. Payer arebe network, asset, recipient, amount na expiry mbere yo gusinya.
4. Ohereza cyangwa urebe transaction; genzura network, source, destination, asset, amount, success na unique reference byigenga. Indexing na receipt creation bibe idempotent kandi bishobore gusubirwamo nyuma y'ikosa.
5. Huza Receipt Vault, notifications na Merchant Assistant n'ubwishyu bwemejwe. Reconciliation, refunds, disputes, rate limits, audit n'ibizamini bya concurrency bikorwe mbere y'amafaranga nyayo.

`intent_anchor` iriho ubu yerekana gusa `require_auth`, expiry na replay guard kuri hash. Hash encoding nyayo, storage lifetime, wallet challenge, privacy review na external audit ntibirakorwa; iyo contract ntigomba gukoreshwa mu kwishyura uko iri.
