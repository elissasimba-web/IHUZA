# IHUZA+ merchant approval plan

Iyi ni plan yo kwemeza ubusabe bw'abacuruzi kuri Stellar testnet. Ntabwo ari approval ikora ubu. `merchant_applications` ibika ubusabe gusa, naho QR resolver ikomeza kwerekana konti isanzwe. Inyandiko ya Governance isaba accountability, privacy, audit na compliance, ariko ntivuga umubare w'abasuzuma cyangwa ibimenyetso bazagenzura. Ibyo byemezo bigomba kubanza kwemeranywaho.

## Icyemezo cy'umucuruzi

- Gusaba kuba umucuruzi ntibihindura `account_kind` kandi ntibifungura checkout.
- Reviewer agenzura ibimenyetso by'ubucuruzi hakurikijwe amabwiriza azemezwa. Ibisobanuro by'irangamuntu cyangwa inyandiko z'ubucuruzi ntibijya muri QR cyangwa kuri Stellar; ahabikwa ibimenyetso hakenera uburenganzira n'igihe bimara bisobanutse.
- Icyemezo kigomba kugaragaza ubusabe, ubwoko bw'icyemezo (`approve`, `reject`, `suspend`), reviewer, igihe n'impamvu cyangwa evidence reference. Amateka y'ibyemezo aba append-only; gukosora bikorwa n'icyemezo gishya.
- Uburenganzira bwa reviewer bugenerwa ibikorwa byo gusuzuma merchant gusa. Ntabwo butanga uburenganzira bwo gusinya payment, guhindura ibiciro by'umucuruzi cyangwa gufata wallet seed.
- QR imwe ya konti ntihinduka iyo merchant yemejwe cyangwa ahagaritswe. Resolver igenzura **approval ikiriho** mbere yo gutanga merchant flow; `accounts.account_kind = 'merchant'` yonyine ntigomba gufungura checkout.

## Uburyo bwa tekiniki buteganyijwe

1. IHUZA+ igena reviewer public addresses za Stellar testnet n'uburyo bwo kuzisimbuza. Reviewer asinya challenge ivuga origin, network, account/application, proposed name digest, decision, nonce n'igihe irangirira. Private key iguma muri wallet ya reviewer.
2. API igenzura allowlist ya reviewer, signature, expiry, single-use nonce n'uko ubusabe butahindutse. Nta self-approval. Uko reviewer yemejwe n'umubare w'imikono isabwa bizagena igihe approval itangira gukora.
3. Mu database transaction imwe, API ibika icyemezo n'ubwoko bushya bwa status ishobora kugenzurwa. Duplicate/replay ntibyongera icyemezo. Suspension ihita ibuza merchant flow, ariko ntikora debit cyangwa ihindura receipts zashize.
4. Catalog, ibiciro na checkout bizashingira kuri status ya merchant yemewe. Umucuruzi ni we uhindura ibiciro bye; reviewer ntiyabihindura. Payment success izashingira ku gihamya cya chain, si approval.

## Ibyemezo bisigaye mbere ya migration

- Ni nde utanga reviewer addresses, ni bangahe basinya approval, kandi ni nde ushobora kuzisimbuza?
- Ni ibihe bimenyetso by'ubucuruzi bigomba kugenzurwa kuri prototype no ku mafaranga nyayo? Ni he bibikwa, ni nde ubisoma, bimara igihe kingana iki?
- Ese rejection ishobora kongera gusabirwa, kandi suspension ifite inzira yo kujurira iyihe?
- Ese display name public ihindurwa ite, kandi ni ryari merchant catalog yemerewe kugaragara?

Nyuma y'ibi byemezo: andika migration nshya itahindura izashyizweho, suzuma SQL na role permissions, hanyuma ugerageze unauthorized reviewer, wrong network/origin, replay, concurrency, suspension na QR flow. Nta approval yo mu production cyangwa payment izatangizwa n'iyi plan yonyine.
