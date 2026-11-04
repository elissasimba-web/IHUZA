# IHUZA+ — Stellar Web3 architecture

## Aho tugeze

Iri shami rifite Soroban `intent_anchor` contract na read-only testnet probe gusa. Nta frontend, API, database schema nshya, wallet connection, QR resolver, chain indexer, payment cyangwa Receipt Vault birimo. Mu iyimurika bigomba kuvugwa uko biri.

Iyi prototype ntikoresha PostgreSQL kugeza ubu. Hazakenerwa database nshya y'amakuru ya off-chain, yubatse ku bisabwa n'iyi architecture; ntituzafata records za database ya kera nk'aho ari amafaranga ya Stellar.

## Aho buri kintu kizaba

| Stellar / Soroban | IHUZA+ API na database | Client y'umukoresha |
| --- | --- | --- |
| Nyiri asset, uburenganzira bw'isinywa, transaction result n'events zikenewe | Identity, role checks, QR reference resolver, merchant catalog, Smart Invoice snapshots, receipts, notifications na audit | QR scan, catalog, invoice review, wallet connection, gusinya no kwerekana status yagenzuwe |

PostgreSQL migrations aracyakenewe ku makuru ya off-chain. Gukoresha database ntibibuza app kuba Web3; icyo tugenzura ni uko server idashobora gukoresha amafaranga y'umukiriya itabisinyiwe kandi ko payment status iva mu bimenyetso bya chain byagenzuwe. Soroban contract ikoreshwa gusa aho amategeko y'ubwishyu asaba programmable logic; native Stellar payment ntigomba gusubirwamo nta mpamvu.

## Umutekano n'uburyo bwo kubaka

1. Hitamo wallet UX na recovery model idaha server private keys. Huze konti na Stellar address ukoresheje challenge ifite network, expiry na signature yagenzuwe.
2. Wubake identity, merchant approval, catalog n'account QR imwe mu API nshya. Ibirimo QR bibe opaque kandi bidashobora gukekwa.
3. Wubake quote na Smart Invoice ibarwa na server, ifite snapshots z'ibiciro n'idempotency. Payer arebe network, asset, recipient, amount na expiry mbere yo gusinya.
4. Ohereza cyangwa urebe transaction; genzura network, source, destination, asset, amount, success na unique reference byigenga. Indexing na receipt creation bibe idempotent kandi bishobore gusubirwamo nyuma y'ikosa.
5. Huza Receipt Vault, notifications na Merchant Assistant n'ubwishyu bwemejwe. Reconciliation, refunds, disputes, rate limits, audit n'ibizamini bya concurrency bikorwe mbere y'amafaranga nyayo.

`intent_anchor` iriho ubu yerekana gusa `require_auth`, expiry na replay guard kuri hash. Hash encoding nyayo, storage lifetime, wallet challenge, privacy review na external audit ntibirakorwa; iyo contract ntigomba gukoreshwa mu kwishyura uko iri.
