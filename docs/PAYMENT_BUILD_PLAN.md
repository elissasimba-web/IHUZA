# IHUZA+ — gahunda yo kubaka ubwishyu

**One Account. Every Service.** IHUZA+ izerekana RWF nk'ifaranga ry'ibanze ry'abakoresha n'abacuruzi. Ihagaze kuri Stellar/Soroban, Rust/Axum, PostgreSQL, Freighter, SEP-53 na QR imwe ihoraho irimo opaque account reference. Ubu hari Smart Invoice `draft` n'ibiciro bya RWF byo kwerekana gusa; nta payment transaction, RWF balance, provider settlement, Receipt Vault cyangwa EBM integration iriho.

## Imbibi z'amafaranga

- **RWF wallet:** balance nyayo izakenera ledger ifite entries zidahindurwa, database transactions, idempotency, reconciliation n'amabwiriza yo kubungabunga amafaranga. Tugomba kubanza kwemeza niba IHUZA+ cyangwa umufatanyabikorwa wemewe ari we ubika amafaranga n'icyo umukiriya atunze. Balance ntishobora kuba field ihindurwa uko bishakiye.
- **Stellar testnet:** XLM izakoreshwa mu igerageza rya P2P gusa. Si RWF, si RWF wallet, si igipimo cy'ivunjisha kandi si uburyo bwo kwishyura Smart Invoice ya RWF. Nta `1 RWF = 1 XLM`; nta conversion cyangwa unofficial RWF token byikorerwa.
- **Payment providers:** RWF payment adapter izakoresha provider wemewe n'ibimenyetso bye bya settlement igihe azaba ahari. Bank na Mobile Money bizagira adapters zihariye; nta n'imwe iri live ubu.
- **Stellar:** native payment cyangwa Soroban ikoreshwa aho ibikorwa bya Web3 bikenewe kandi byemewe. Buri value movement isaba uburenganzira bwa wallet nyirayo. Backend ntibika private key/seed; SEP-53 login si umukono w'ubwishyu.
- **CBDC:** adapter itandukanye izategereza uburenganzira n'ibisobanuro bya tekiniki byemewe na BNR. Ubushakashatsi cyangwa proof of concept ntibivuga ko IHUZA+ ifite access.
- **EBM:** invoice ya IHUZA+ n'inyemezabwishyu bibikwa off-chain. EBM adapter izakorwa nyuma y'ibisabwa na RRA; invoice yacu ntihita iba iya EBM.

## QR imwe na payment intent

QR y'account iguma kuba opaque reference imwe, nta PIN, secret, amount cyangwa invoice data iyijyamo. Server isobanura reference n'ubwoko bwa konti. Person-to-person, person-to-merchant na merchant-to-merchant bazakoresha iyo account identity, ariko provider n'amategeko ya buri flow bigenzurwe ukwabyo. Invoice ID ishobora kuba payment reference; ntisaba QR ya kabiri ya konti.

Payment intent ibika immutable sender, recipient, currency/asset, amount mu units nyazo, provider/network, expiry, unique reference na idempotency key. Sender abona details zose mbere yo gusinya muri Freighter cyangwa kwemeza provider request. `submitted` cyangwa hash itanzwe n'umukiriya ntibiba `paid`. Adapter yemeza result y'ukuri iva kuri provider cyangwa kuri Stellar ledger, igahuza source, destination, currency/asset, amount, reference na transaction success. Iyo igihamya kibuze, status iguma `verifying` cyangwa ijya `failed/review`.

Reconciliation ibuza transaction imwe kwishyura intents ebyiri cyangwa intent imwe gutanga receipts ebyiri. Result imaze kwemezwa, system ikora record y'ubwishyu, fagitire ya IHUZA+ n'inyemezabwishyu idempotently, ikabika muri Receipt Vault y'impande zombi, ikohereza notifications. Nta muntu usabwa gukanda ngo yemeze ko amafaranga yageze. Refund na dispute byandikwa ukwayo; record ya mbere ntihindurwa bucece. Amakuru yihariye aguma off-chain.

## Uko tuzabikurikiranya nyuma yo kwemeza iyi plan

1. **P2P XLM testnet lab:** payment intent itandukanye na Smart Invoice ya RWF; native XLM transaction ifite recipient, amount, network, fees na expiry bigaragara mbere y'uko sender asinyira muri Freighter. Koresha test accounts gusa. Nta merchant checkout cyangwa mainnet.
2. **Verification na receipts z'igerageza:** ohereza signed XDR kuri testnet, ugenzure chain result na operation byigenga, ubike status na receipt idempotently. Gerageza failures, duplicates, network outage, wallet switch, wrong destination/amount n'uko system isubukura nyuma yo gutakaza connection.
3. **RWF wallet/provider design:** hitamo custody/safeguarding model, provider n'uburenganzira; tegura double-entry ledger, balance ikomoka kuri entries, concurrent transactions, idempotency, reconciliation, limits, audit, recovery na dispute handling. Ntukoreshe XLM testnet nk'aho ari RWF.
4. **Merchant RWF checkout:** huza Smart Invoice snapshots na RWF payment request; provider settlement ni yo ifungura `confirmed`, fagitire na Receipt Vault. Merchant ni we ugumana ibiciro bye. Delivery KYC ni flow yihariye.
5. **Izindi adapters:** bank/Mobile Money, CBDC na EBM byongerwe gusa iyo ubufatanye, specifications, certification n'ibizamini bihari. Soroban ikoreshwe aho programmable behavior ikenewe; ntisimbure native payment cyangwa provider settlement nta mpamvu.

## Igenzura ribanziriza code

- Mbere ya migration nshya, andika plan ya schema, suzuma SQL, permissions n'uko 001–007 zidahinduwe; shyira migration kuri `ihuza_web3_testnet` gusa nyuma y'igenzura.
- P2P testnet izasaba tests za wrong signer/network/recipient, amount/asset mismatch, expired intent, duplicate hash, concurrent retries, failed/pending transaction, chain API outage na idempotent receipt. Wallet signing na chain verification nyabyo bigeragezwe; mock yonyine ntihagije.
- RWF wallet nyayo izasaba isuzuma ry'amategeko, licensing/ubufatanye, safeguarding na security audit mbere yo gufata amafaranga y'abakiriya. Nta mainnet cyangwa amafaranga nyayo bitangizwa n'iyi plan.

Amakuru y'ibanze: BNR payment systems https://www.bnr.rw/paymentsystems ; BNR CBDC proof of concept https://www.bnr.rw/documents/Rwanda_CBDC_PoC_Report_CbiVoqg.pdf ; RRA EBM https://www.rra.gov.rw/en/about-ebm .
