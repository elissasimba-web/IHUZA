# IHUZA+ — ibyo twemeranyijwe

Iyi nyandiko ivuga imyitwarire ya product izubakwa. Ntivuga ko ibyo byose byamaze gukorwa.

## Konti, wallet na QR

- Buri konti igira QR imwe ihoraho, irimo opaque reference gusa. Nta secret, igiciro, balance cyangwa invoice iyijyamo.
- Uwayiscan-nye abona flow igenwa n'ubwoko bwa konti n'ibyemejwe na server: merchant checkout ku mucuruzi wemewe, person transfer ku mukiriya usanzwe. Merchant ntiyubaka sale ascan-a QR y'umukiriya usanzwe.
- Nyiri wallet ni we usinya buri gikorwa cyimura value. App login cyangwa ADMIN ntibisimbura uburenganzira bwa wallet kandi ntibika seed/private key.
- Serivisi za mbere zikora online gusa. Nta kwishyura offline cyangwa automatic debit.

## Merchant checkout

- Umucuruzi ni we wandika ibicuruzwa n'ibiciro bye. Umukiriya ahitamo item na quantity; server yongera kubara ibiciro na total.
- Smart Invoice ibika immutable snapshots z'izina ry'ikicuruzwa, quantity, unit price, total, currency, merchant na customer aho azwi. Nta invoice QR ya kabiri.
- Umukiriya yemera ibyo ahisemo mbere yo gusinya ubwishyu. Merchant ntasabwa kubanza kwemeza urutonde kugira ngo transaction ikorwe.
- Payment success ishingira ku gisubizo cya Stellar cyagenzuwe byigenga; hash yoherejwe cyangwa notification ntibihagije. Payment ntivuga ko ibicuruzwa byatanzwe.
- Nyuma y'ubwishyu bwemejwe, inyemezabwishyu ya IHUZA+ ijya muri Receipt Vault y'umukiriya n'umucuruzi. Refund cyangwa adjustment yandikwa ukwayo kandi igahuza n'inyandiko ya mbere; nta guhindura inyemezabwishyu ya kera bucece.
- Iyo merchant akeka ko item itishyuwe, abanza kugenzura ibyo yatanze; ashobora gusaba adjustment. Umukiriya ashobora kuyemera cyangwa kuyijyaho impaka. Nta debit ikorwa nta bwemere bwe.

## Merchant Assistant n'amakuru

- Suppliers, procurement, stock movements, products, sales, revenue na reports bigira amateka ashobora kugenzurwa. Profit ntibarwa nk'aho ari revenue; costing igomba kubanza gusobanurwa.
- Amakuru yihariye, catalog, invoice items, receipts na audit records abikwa off-chain mu buryo bugenzurwa n'uburenganzira bwa buri mukoresha. Ibyo kuri chain bigomba kuba bike kandi bikenewe mu kugenzura ukuri.
- Assistant cyangwa AI ishobora gusobanura records zemejwe, ariko ntishobora kwemeza ubwishyu, guhindura total, cyangwa kwikura amafaranga muri wallet.

## Ibyo tutavuga ko bihari

Nta RWF asset/issuer, amafaranga nyayo, bank, MTN, Airtel, CBDC cyangwa EBM integration yemejwe. Inyemezabwishyu ya IHUZA+ si fagitire ya EBM. Ubushakashatsi bw'amategeko, impushya, safeguarding na reconciliation birakenewe mbere y'amafaranga nyayo.
