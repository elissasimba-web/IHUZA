# IHUZA+ merchant approval plan

Iyi ni plan n'uko prototype ya review ihagaze kuri Stellar testnet. Nyiri project yemeje **reviewers 3 batandukanye, threshold ya 2-of-3**. `merchant_applications` ibika izina ry'ubucuruzi; `merchant_profile_versions` ibika izina ry'uwasabye n'irya company iyo rihari. `merchant_profile_votes` ibika imikono ya reviewers kuri profile yose. Votes za kera zo muri `merchant_approval_votes` ntizongera kubarwa. Reba `docs/MERCHANT_VERIFICATION_PLAN.md`. Iyi review si KYC cyangwa business registration verification.

## Icyemezo cy'umucuruzi

- Gusaba kuba umucuruzi ntibihindura `account_kind` kandi ntibifungura checkout.
- Reviewer abona amazina y'ibanze yatanzwe, agasinyira iyo profile ya testnet. Nta nyandiko y'ubucuruzi cyangwa irangamuntu ibikwa ubu; delivery KYC na EBM bizagira amabwiriza yabyo nyuma.
- Vote ya `approve` ibika profile ID na digest yayo, reviewer, reviewer set digest, nonce, igihe n'umukono. Amateka y'ibyemezo aba append-only. `reject` na `suspend` ntibirubakwa; bizasaba policy n'amateka yabyo.
- Uburenganzira bwa reviewer bugenerwa ibikorwa byo gusuzuma merchant gusa. Ntabwo butanga uburenganzira bwo gusinya payment, guhindura ibiciro by'umucuruzi cyangwa gufata wallet seed.
- QR imwe ya konti ntihinduka iyo merchant yemejwe cyangwa ahagaritswe. Resolver igenzura **approval ikiriho** mbere yo gutanga merchant flow; `accounts.account_kind = 'merchant'` yonyine ntigomba gufungura checkout.

## Uburyo bwa tekiniki buteganyijwe

1. IHUZA+ igena reviewer public addresses **eshatu** za Stellar testnet muri config ya server. Zigomba kuba valid kandi zitandukanye; iyo config ibuze cyangwa ifite ikibazo, approval irafungwa. Reviewer asinya ubutumwa bwa SEP-53 buvuga origin, network, account, profile ID na digest y'amazina yose, reviewer set digest, decision `approve`, nonce n'igihe kitarenza iminota 5. Private key iguma muri wallet ya reviewer.
2. API igenzura session ya reviewer, allowlist, signature, expiry, nonce unique n'uko profile itahindutse. Reviewer ntiyiyemeza ubwe. Buri reviewer atanga vote imwe kuri profile version; **votes 2 za addresses zitandukanye mu itsinda rya 3** ni zo zemeza merchant.
3. Vote ibikwa append-only muri database. QR resolver yongera kugenzura signatures ebyiri n'uko zituruka mu reviewer set iriho mbere yo kwerekana merchant status. Guhindura reviewer set bifunga approvals za set ishaje kugeza reviews nshya zibayeho. Duplicate/replay ntibyongera vote. Suspension na rejection bizagira inzira yihariye nyuma yo kwemeza policy yabyo.
4. Catalog, ibiciro na checkout bizashingira kuri status ya merchant yemewe, ariko ubu QR ikomeza kugaragaza `flow: unavailable` n'iyo votes 2 zimaze kuboneka. Umucuruzi ni we uzahindura ibiciro bye; reviewer ntiyabihindura. Payment success izashingira ku gihamya cya chain, si approval.

## Ibisigaye mbere yo gukoresha approval hanze ya testnet

- Ni nde utanga reviewer public addresses eshatu kandi ni nde ushobora kuzisimbuza? Kuri prototype, addresses zishyirwa muri local config na nyiri project; nta private key ijya muri repository.
- Ni ibihe bimenyetso by'ubucuruzi bigomba kugenzurwa kuri prototype no ku mafaranga nyayo? Ni he bibikwa, ni nde ubisoma, bimara igihe kingana iki?
- Ese rejection ishobora kongera gusabirwa, kandi suspension ifite inzira yo kujurira iyihe?
- Ese display name public ihindurwa ite, kandi ni ryari merchant catalog yemerewe kugaragara?

Migration `004_merchant_approval_votes.sql` ni iya testnet gusa. `IHUZA_REVIEWER_ADDRESSES` igomba kuba irimo public addresses eshatu zemewe kandi zitandukanye; iyo ibuze, review endpoint irafungwa kandi QR ikomeza gufata konti nk'isanzwe. Igeragezwa rya local rikoresha keys z'igihe gito muri memory gusa. Nta approval yo mu production cyangwa payment izatangizwa n'iyi prototype.

Web ubu ibaza API niba session iriho ari iya reviewer mbere yo kwerekana review panel. Ibi bifasha UX gusa; uburenganzira bw'ukuri bugenzurwa nanone kuri `GET/POST /api/merchant-review/{reference}`.
