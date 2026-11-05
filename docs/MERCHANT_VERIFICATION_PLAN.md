# IHUZA+ — merchant profile ya testnet

Nyiri project yahisemo koroshya merchant registration ya testnet: izina ry'uwasabye, izina ry'ubucuruzi, n'izina rya company niba ayifite. Nta TIN, nimero y'iyandikisha, certificate, ID scan cyangwa KYC bisabwa ku mucuruzi usanzwe muri iyi prototype. Reviewer approval ya 2-of-3 yemeza **profile yatanzwe kuri testnet gusa**; ntivuga ko ubucuruzi bwemejwe na RDB/RRA.

Delivery no gutumiza ibicuruzwa imbere mu gihugu bizasaba company yanditswe n'isuzuma rya KYC ryihariye igihe icyo gice kizubakwa. Nta delivery flow ifungurwa na merchant approval isanzwe. TIN, ibyangombwa byemewe n'uburyo bwo kubihuza na EBM bizategurwa igihe EBM integration yemewe izubakwa. Nta EBM integration iriho ubu.

## Migration 006: plan yasuzumwe mbere yo kuyikoresha

- Ongera `merchant_profile_versions` muri `ihuza_web3_testnet` gusa: `id`, `account_id`, `version`, `applicant_name`, `company_name` ishobora kubura, `created_at`. `merchant_applications.proposed_name` iguma ari izina ry'ubucuruzi. Nta table ya kera ihindurwa cyangwa ngo isibwe. Profile ya mbere iba version 1; guhindura amakuru nyuma bizasaba version nshya n'isuzuma rishya, ntihindura iyashize.
- Ongera `merchant_profile_votes` ya append-only ifite umukono wa reviewer, profile ID na digest y'izina ry'ubucuruzi n'andi mazina, reviewer set digest, nonce n'igihe. Runtime database role ihabwa `SELECT/INSERT` gusa. Migration igira guard ya database ya testnet.
- Votes za `004_merchant_approval_votes.sql` zasinyiraga izina gusa. Zisigara mu mateka ariko **ntizongera gufungura merchant QR cyangwa catalog**. Konti ifite application ya kera ishobora kuzuza profile ya version 1, igategereza votes nshya 2 muri 3. QR reference imwe ihoraho ntihinduka.
- API ikura account ID muri session, ikagenzura amazina n'uko company name itangwa iyo ihari. Owner na reviewer wemewe gusa ni bo basoma applicant/company names; public QR/catalog ntibisohora ayo makuru. Reviewer abona amazina mbere yo gusinya ubutumwa bushya bwa SEP-53 butandukanye n'ubwa kera.
- Nta wallet seed, payment, transaction hash, EBM cyangwa delivery authorization muri iyi migration.

## Igenzura

1. Mbere yo `-Apply`: suzuma SQL, database guard, permissions na `git diff --check`; wemeze ko migration 001–005 zidahinduwe.
2. Nyuma: genzura migration info, old application ishobora kongeraho profile, old vote ntikora, unauthorized read/write, company optional, duplicate/mismatch, self-approval, quorum, replay, reviewer rotation na QR/catalog fail closed.
3. UI igomba kwerekana neza ko `approved` ari approval ya testnet profile, si KYC cyangwa EBM.

## Amasoko yo guteguraho EBM/KYC nyuma

- RDB/ORG: https://org.rdb.rw/business-registration/ — registered company n'individual enterprise ni ibyiciro bitandukanye.
- RRA: https://www.rra.gov.rw/fileadmin/user_upload/announce/itangazo_tin_17_feb.pdf — certificate ya company yanditswe irimo TIN; ntisaba TIN certificate itandukanye.
