# IHUZA+ — payment build plan (testnet)

Iyi plan itegura code y'ubwishyu, ariko ntabwo ivuga ko transaction cyangwa verification byamaze kubakwa. API na database biracyafite Smart Invoice `draft` gusa. Ntihagire `paid` cyangwa fagitire y'ubwishyu itangwa mbere yo kugenzura chain result byigenga.

## Icyemezo cy'asset gikenewe

- Catalog y'umucuruzi ifite `RWF` yo kwerekana igiciro; si Stellar asset. Nta mpamvu yo guhindura 1000 RWF ngo bibe 1000 XLM. Merchant payment izategereza asset y'igerageza, uburyo bwo kubara exchange rate/quote, fee na expiry byemejwe neza.
- Person-to-person transfer ya XLM ya testnet ishobora kubakwa ukwihariye, ifite amount ya XLM yanditswe na sender, nta Smart Invoice ya RWF iyivanzwamo. Uko kwigenga ntibivuga ko merchant checkout yarangiye.

## Ibice bihuriweho by'umutekano

1. Payment intent ibika sender, recipient wallet address, network, asset, amount mu units nyazo, expiry, unique reference na idempotency key. Server ntibika seed/private key. Session yonyine ntihagije kwimura value.
2. UI yerekana details zose mbere y'uko sender yemeza transaction muri Freighter. Transaction XDR yubakwa ku network, recipient, asset na amount byavuzwe muri intent. Nta guhindura details nyuma yo gusinya.
3. Icyo wallet yohereje cyangwa hash yanditswe kiba `submitted/verifying` gusa. Backend isoma ledger/operations byigenga, ikagenzura transaction success, network, source, destination, asset, amount na unique reference. Wrong/missing result ntiba `paid`.
4. Reconciliation ikoreshwa kenshi kandi idempotently; transaction imwe ntishobora kwishyura intents ebyiri, intent imwe ntishobora gutanga receipts ebyiri. Igihe chain API itaboneka, status iguma `verifying`.
5. Chain imaze kwemezwa, system ikora fagitire ya IHUZA+ n'inyemezabwishyu muri database transaction imwe, ikabika mu Receipt Vault y'impande zombi kandi ikohereza notifications. Nta muntu usabwa kwemeza transaction nyuma yayo. Fagitire ya EBM ni integration yihariye izaza nyuma.

## Igenzura rizakenerwa

- Testnet transaction nyayo ifite wallet signing y'umukoresha; wrong signer, network, recipient, asset, amount, expiry, duplicate hash, retry, concurrent submissions na chain API outage bigeragezwe.
- DB migrations z'ubwishyu zitegurwe kandi zisuzumwe mbere yo kuzikoresha kuri `ihuza_web3_testnet`; migrations 001–007 ntizihindurwe.
- Nta mainnet, amafaranga nyayo, RWF asset, CBDC cyangwa EBM bivugwa ko byubatswe mbere y'igenzura n'uburenganzira bwihariye.
