# IHUZA+ — kwemeza ko amafaranga yageze

Iyi ni gahunda y'ubwishyu buzubakwa nyuma ya Smart Invoice draft. Nta payment execution cyangwa chain verification iri muri code ubu. `RWF` yo muri catalog ni igiciro cyo kwerekana; ntiharatoranywa asset yo kwishyura kuri Stellar, uburyo bwo guhindura igiciro mu asset, fees cyangwa umufatanyabikorwa. Ibyo bigomba kubanza kwemezwa mbere yo gukora payment request.

## Uko status izagenda

1. **Draft:** server yabaze invoice, ariko nta mafaranga yoherejwe.
2. **Awaiting signature:** uwayohereza abona uwo yoherereza, asset, amount, network, fees, invoice reference n'igihe ntarengwa; abyemera muri wallet ye. Gusinya ni uburenganzira bwo kugerageza transaction, si gihamya ko yageze.
3. **Submitted / verifying:** hash cyangwa igisubizo cya wallet birabikwa kugira ngo bigenzurwe; nta ruhande rwerekwa `paid` kuri iyi ntambwe.
4. **Confirmed:** indexer/API isoma transaction kuri Stellar testnet byigenga, igenzura ko iri muri ledger kandi yatsinze, operation y'ubwishyu ikahuza source, destination, asset, amount na reference byari byemejwe. Imikorere ya replay/reconciliation ibuza invoice imwe kwishyurwa inshuro ebyiri cyangwa transaction imwe kwishyura invoices ebyiri. Iyo ibimenyetso bituzuye, status iguma `verifying` cyangwa ijya `failed/review`. Nta muntu usabwa gukanda ngo yemeze transaction.
5. **Fagitire na Receipt Vault:** ako kanya nyuma ya `confirmed`, system ikora fagitire ya IHUZA+ y'ubwishyu yifashishije draft snapshot n'ikimenyetso cya chain, ikayibika idempotently muri Receipt Vault y'uwayohereje n'uwakiriye. Bombi babona status imwe n'iyo fagitire; notification ibamenyesha gusa. Kubura cyangwa gutinda kwa notification ntibihindura ubwishyu cyangwa ngo bitinde kubika fagitire.

Nta gikorwa cya “Nabonye ubutumwa” cyangwa ukwemeza kw'uwakiriye gisabwa. Niba hari utemeranya n'ibimenyetso by'ubwishyu, afungura dispute/reconciliation; record n'inyemezabwishyu ya mbere ntibihindurwa bucece. Ibi bikora kuri merchant checkout no kohereza amafaranga hagati y'abantu, mu gihe buri flow izaba yubatswe. Fagitire ya EBM izongerwa gusa igihe integration yemewe izaba yubatswe; fagitire ya IHUZA+ ntihinduka iya EBM ku bwo kuba amafaranga yemejwe.

## Ibisabwa mbere yo gukora code y'ubwishyu

- Hitamo testnet asset nyayo, issuer iyo ikenewe, exchange/quote model ya RWF display price, fee payer na trustline UX. Ntituvuga ko RWF token cyangwa CBDC bihari.
- Tegura payment intent idahinduka ifitanye isano na invoice cyangwa person transfer, idempotency, expiry, unique on-chain reference n'uburyo bwo kuyigarura nyuma yo gutakaza connection.
- Genza testnet transaction zitandukanye: success, failure, pending, duplicate hash, wrong destination/asset/amount, multiple operations, network mismatch, reorg/retry n'ibura rya notification. Payment success ntiva kuri hash yoherejwe n'umukiriya.

Stellar references: https://developers.stellar.org/docs/data/apis/horizon/api-reference/resources/accounts na https://developers.stellar.org/docs/data/apis/horizon/api-reference/resources/effects
