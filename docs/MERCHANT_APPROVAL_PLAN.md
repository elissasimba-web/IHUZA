# IHUZA+ merchant approval plan

Iyi ni plan n'uko prototype ya review ihagaze kuri Stellar testnet. Nyiri project yemeje **reviewers 3 batandukanye, threshold ya 2-of-3**. `merchant_applications` ibika ubusabe; `merchant_approval_votes` ibika imikono ya reviewers. Inyandiko ya Governance isaba accountability, privacy, audit na compliance; ibimenyetso by'ubucuruzi byo ku mafaranga nyayo bizasobanurwa ukundi.

## Icyemezo cy'umucuruzi

- Gusaba kuba umucuruzi ntibihindura `account_kind` kandi ntibifungura checkout.
- Reviewer agenzura ibimenyetso by'ubucuruzi hakurikijwe amabwiriza azemezwa. Ibisobanuro by'irangamuntu cyangwa inyandiko z'ubucuruzi ntibijya muri QR cyangwa kuri Stellar; ahabikwa ibimenyetso hakenera uburenganzira n'igihe bimara bisobanutse.
- Vote ya `approve` ibika application, reviewer, reviewer set digest, nonce, igihe n'umukono. Amateka y'ibyemezo aba append-only. `reject` na `suspend` ntibirubakwa; bizasaba policy n'amateka yabyo.
- Uburenganzira bwa reviewer bugenerwa ibikorwa byo gusuzuma merchant gusa. Ntabwo butanga uburenganzira bwo gusinya payment, guhindura ibiciro by'umucuruzi cyangwa gufata wallet seed.
- QR imwe ya konti ntihinduka iyo merchant yemejwe cyangwa ahagaritswe. Resolver igenzura **approval ikiriho** mbere yo gutanga merchant flow; `accounts.account_kind = 'merchant'` yonyine ntigomba gufungura checkout.

## Uburyo bwa tekiniki buteganyijwe

1. IHUZA+ igena reviewer public addresses **eshatu** za Stellar testnet muri config ya server. Zigomba kuba valid kandi zitandukanye; iyo config ibuze cyangwa ifite ikibazo, approval irafungwa. Reviewer asinya ubutumwa bwa SEP-53 buvuga origin, network, account/application, proposed name digest, reviewer set digest, decision `approve`, nonce n'igihe kitarenza iminota 5. Private key iguma muri wallet ya reviewer.
2. API igenzura session ya reviewer, allowlist, signature, expiry, nonce unique n'uko ubusabe butahindutse. Reviewer ntiyiyemeza ubwe. Buri reviewer atanga vote imwe kuri application imwe; **votes 2 za addresses zitandukanye mu itsinda rya 3** ni zo zemeza merchant.
3. Vote ibikwa append-only muri database. QR resolver yongera kugenzura signatures ebyiri n'uko zituruka mu reviewer set iriho mbere yo kwerekana merchant status. Guhindura reviewer set bifunga approvals za set ishaje kugeza reviews nshya zibayeho. Duplicate/replay ntibyongera vote. Suspension na rejection bizagira inzira yihariye nyuma yo kwemeza policy yabyo.
4. Catalog, ibiciro na checkout bizashingira kuri status ya merchant yemewe, ariko ubu QR ikomeza kugaragaza `flow: unavailable` n'iyo votes 2 zimaze kuboneka. Umucuruzi ni we uzahindura ibiciro bye; reviewer ntiyabihindura. Payment success izashingira ku gihamya cya chain, si approval.

## Ibisigaye mbere yo gukoresha approval hanze ya testnet

- Ni nde utanga reviewer public addresses eshatu kandi ni nde ushobora kuzisimbuza? Kuri prototype, addresses zishyirwa muri local config na nyiri project; nta private key ijya muri repository.
- Ni ibihe bimenyetso by'ubucuruzi bigomba kugenzurwa kuri prototype no ku mafaranga nyayo? Ni he bibikwa, ni nde ubisoma, bimara igihe kingana iki?
- Ese rejection ishobora kongera gusabirwa, kandi suspension ifite inzira yo kujurira iyihe?
- Ese display name public ihindurwa ite, kandi ni ryari merchant catalog yemerewe kugaragara?

Migration `004_merchant_approval_votes.sql` ni iya testnet gusa. `IHUZA_REVIEWER_ADDRESSES` igomba kuba irimo public addresses eshatu zemewe kandi zitandukanye; iyo ibuze, review endpoint irafungwa kandi QR ikomeza gufata konti nk'isanzwe. Igeragezwa rya local rikoresha keys z'igihe gito muri memory gusa. Nta approval yo mu production cyangwa payment izatangizwa n'iyi prototype.
