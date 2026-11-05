# IHUZA+ — merchant verification ya testnet

Iyi plan ishingiye ku byo nyiri project yemeje: izina ry'ubucuruzi, uwabusabye, aho bukorera, nimero y'iyandikisha cyangwa TIN, n'izina ryemewe rya company iyo ari company. Delivery no gutumiza mu gihugu bizagira policy yihariye nyuma; ntibifungurwa n'iyi verification. Iyi ni prototype ya testnet, si ukwemezwa na RDB/RRA cyangwa impushya zo kwakira amafaranga nyayo.

## Ibyo merchant atanga

1. Izina rigaragara ku bakiriya (`display_name`, riri muri `merchant_applications.proposed_name`).
2. Ubwoko bw'ubucuruzi: individual enterprise cyangwa registered company. Company itanga `registered_company_name`; individual enterprise ntisabwa guhimba izina rya company.
3. Izina ry'uwasabye (`applicant_name`) n'aho ubucuruzi bukorera (`operating_location`). Ibi ntibijya muri QR cyangwa public catalog.
4. Nimero y'iyandikisha / TIN imwe, yanditswe uko iri ku cyemezo cy'iyandikisha. Nta “TIN certificate” ya kabiri isabwa kuri company yanditswe. Reviewer agomba kugenzura ko nimero n'izina bihura n'icyemezo.
5. Uko kopi y'icyemezo (PDF/ifoto) yatangwa muri system cyangwa igenzurwa hanze yayo biracyategereje icyemezo cya nyiri project. Nta file upload izongerwa tutarategura uburenganzira bwo kuyisoma, uburyo bwo kuyibika n'igihe izamura.

## Ingaruka kuri approval iriho

Votes zo muri `merchant_approval_votes` zasinyiraga izina gusa. Ntizishobora kwitirirwa ko zagenzuye nimero, company name, aho bukorera cyangwa certificate. Migration nshya izaba additive; ntihindura `003`/`004` yashyizweho. Igihe verification nshya izatangira, approval ya catalog/merchant QR izasaba **votes nshya 2 muri 3** ziboshye kuri digest y'amakuru n'ibimenyetso byasuzumwe. Konti idafite ibyo bimenyetso izaba `pending` (fail closed). QR reference imwe ntihinduka. Nta wallet seed, payment cyangwa transaction hash muri verification.

## Privacy na audit

- Application owner na reviewer wemewe ni bo gusa basoma amakuru y'ubucuruzi atari public. Public QR resolver itanga status gusa. API ntisohora nimero y'iyandikisha/TIN mu QR cyangwa public catalog.
- Reviewer abanza kureba izina n'amakuru agiye kwemeza, hanyuma agasinyira ubutumwa bwa SEP-53 burimo application digest, network, decision, nonce n'igihe. Umukono umwe = vote imwe; 2 muri 3 ni zo zemeza kuri testnet.
- Gukosora amakuru nyuma ya votes bisaba version nshya n'isuzuma rishya; ntihindura ibyagezweho bucece. Rejection, suspension, appeal n'igihe ibyangombwa bibikwa bizasobanurwa mbere yo gufungura ayo ma flows.
- Delivery / ordering bizasaba company yanditswe n'ibyangombwa byayo hakurikijwe policy izemezwa igihe icyo gice kizubakwa. Merchant verification y'ibanze ntiyihita itanga uburenganzira bwa delivery.
- Amakuru y'iyandikisha ashobora gufasha mapping ya EBM mu gihe izaba yahujwe byemewe; ubu nta EBM integration cyangwa fagitire ya EBM bihari.

## Gahunda yo gushyira mu bikorwa

1. Emeza niba kopi y'icyemezo izabikwa muri IHUZA+ cyangwa izagenzurwa n'abasuzuma hanze ya system. Hitamo aho ibikwa, access na retention niba izoherezwa.
2. Andika migration nshya ya testnet y'ibyemezo n'amateka ya versions, uyisuzume mbere yo kuyikoresha. Reba ko database ari `ihuza_web3_testnet` kandi ko role ya API ifite uburenganzira buke.
3. Ongera form ya merchant, reviewer preview, signature ya version nshya, na approval gating. Votes za kera zigume mu mateka ariko ntizemererwe gufungura merchant status nshya.
4. Gerageza missing/invalid fields, company vs individual, unauthorized reads, self approval, quorum, replay, changed evidence, old votes, reviewer rotation na QR/catalog fail closed.

## Amasoko y'igenzura

- RDB/ORG: https://org.rdb.rw/business-registration/ — registration ya company n'individual enterprise.
- RDB/ORG requirements: https://org.rdb.rw/business-registration/requirements/ — ibisabwa biratandukana bitewe n'ubwoko bw'ubucuruzi.
- RRA: https://www.rra.gov.rw/fileadmin/user_upload/announce/itangazo_tin_17_feb.pdf — registration certificate ya company yanditswe irimo TIN; ntisaba certificate ya TIN itandukanye.
