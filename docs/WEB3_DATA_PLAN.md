# IHUZA+ — gahunda ya database nshya ya testnet

Migration `006_merchant_profile_versions.sql` yongeraho profile versions na votes zibohwa kuri profile yose. `004` votes z'izina rimwe ziguma mu mateka gusa. Merchant usanzwe atanga amazina y'ibanze; KYC ya delivery na EBM ntibirubakwa.

Migration `007_smart_invoice_drafts.sql` ibika invoice drafts n'imirongo ifite snapshots z'ibiciro; reba `docs/SMART_INVOICE_PLAN.md`. Nta payment status yemejwe cyangwa EBM invoice biyivamo.

Iyi ni gahunda n'amateka yo kubaka API na database ya Web3 testnet. Migrations `001_identity.sql` kugeza kuri `007_smart_invoice_drafts.sql` zashyizwe muri `ihuza_web3_testnet` na `sqlx`. API ya localhost itanga challenge ikanagenzura proof, ikarema cyangwa igasoma konti muri transaction, ikanatanga session. Frontend ihuza Freighter kuri iyo API. `003` ibika merchant applications, `004` ikabika votes za reviewers, `005` ikabika catalog n'ibiciro byanditswe n'umucuruzi; nta payment cyangwa wallet funds bibikwa hano.

## Gahunda yasuzumwe ya `002_sessions.sql`

- Ongera `account_sessions`: account ID, SHA-256 hash ya token ya random 256-bit, igihe yatangiwe, expiry itarenza amasaha 12, na `revoked_at`. Nta token mbisi cyangwa wallet secret ijya muri database. Token mbisi ihabwa browser muri `HttpOnly; SameSite=Strict` cookie; `Secure` izasabwa kuri HTTPS ya production. Iyi prototype ikora kuri localhost HTTP gusa.
- Ongera `session_events` ya append-only (`created`, `revoked`) kugira ngo gufungura no gufunga session bisigire audit. Runtime role yemerewe SELECT/INSERT kuri events, SELECT/INSERT na UPDATE ya `revoked_at` gusa kuri sessions, nta DELETE. Migration igira database-name guard nk'iya mbere; `001_identity.sql` ntihindurwa.
- Nyuma ya proof yemejwe, kora session muri transaction imwe n'account/challenge/audit. `GET /api/session` isoma cookie, igenzura hash, expiry na revocation; `POST /api/session/logout` ifunga session ikanahanagura cookie. POST requests zigenzura exact local Origin. Session ntisimbura umukono wa wallet ku mafaranga.
- Mbere yo gukoresha migration: genzura SQL, ingaruka ku role, ko database ari `ihuza_web3_testnet`, n'uko API tests zikora. Nyuma yo kuyikoresha: genzura migration info, proof→session→logout→replay, ndetse ko database ibika hash y'uburebure bwa bytes 32 gusa. Nta migration yasibwe cyangwa yongeye kwandikwa.

## Gahunda ya `003_merchant_applications.sql`

- Ubusabe bwo kuba umucuruzi bubikwa off-chain muri `merchant_applications`: `account_id` imwe, izina umusaba yatanze, igihe yabisabiye. Iyo bugitangwa buba `pending`; votes 2 zemewe zishobora kubwerekana nka `approved` kuri testnet. Ubusabe ubwabwo ntibuhindura `accounts.account_kind`, QR cyangwa uburenganzira bwo kwakira checkout. Izina ntiryemejwe n'ubusabe bwonyine.
- Gukora no gusoma ubusabe bisaba session ya konti nyirayo. API ikura `account_id` muri session yagenzuwe, si mu byo umukoresha yohereje. Ubusabe bumwe kuri konti; gusubiramo izina rimwe ntibyongera record, guhindura izina bisaba inzira izasuzumwa nyuma. Runtime role ihabwa `SELECT` na `INSERT` gusa; nta `UPDATE` cyangwa `DELETE`.
- Igihe `003` yakorwaga nta reviewer endpoint yariho. Ubu `004` na API ya 2-of-3 birahari kuri testnet. Ibimenyetso bisabwa, rejection na suspension biracyakeneye policy. Wallet ya customer ikomeza kugenzurwa na nyirayo.
- Migration igira guard ya `ihuza_web3_testnet`; ntihindura migrations zashyizweho. Isuzuma ryakozwe kuri session yemewe/itarimo, duplicate, isolation hagati ya konti ebyiri, na QR ya konti itaremezwa igaragaza `person_transfer_preview`. Nyuma ya votes 2 zemewe, QR igaragaza merchant status ariko checkout ikaguma `unavailable`.

## Ahazubakwa

- Approval ya testnet: reviewer public addresses eshatu zitandukanye muri config; umukono wa SEP-53 kuri profile version yose, reviewer set, nonce n'igihe. Votes ebyiri mu itsinda ririho zigaragaza merchant status, ariko checkout irafunze. Reba `docs/MERCHANT_APPROVAL_PLAN.md`.

- Database yihariye `ihuza_web3_testnet` ifite connection string yayo. Mainnet izakenera database n'amabwiriza byayo ukwabyo.
- Umukoresha atangira yemeza ko afite urufunguzo rwa Stellar testnet address akoresheje SEP-53. Konti nshya n'ihuriro rya wallet bikorerwa muri transaction imwe nyuma yo kugenzura challenge. Konti iriho ishobora kongera kwinjira ikoresheje address imaze guhuzwa; uburyo bwo kongeramo indi wallet buzabanza kugira amabwiriza yihariye.
- QR imwe ya konti igira random opaque reference itajya ivugururwa kubera guhindura wallet. Ntijyamo address, secret, igiciro cyangwa balance.

## Imbonerahamwe z'ibanze ziteganyijwe

| Table | Iby'ingenzi | Inzitizi |
| --- | --- | --- |
| `accounts` | UUID, account type, QR reference, created time | QR reference unique kandi random |
| `wallet_links` | account UUID, network, Stellar address, linked time, status | `(network, address)` unique; nta private key |
| `wallet_challenges` | challenge ID, network, address, nonce, issued/expiry time, consumed time, purpose | nonce unique; expiry ntirenza iminota 5; single use |
| `security_events` | event ID, account/challenge references, outcome, time | audit idahindurwa bucece; nta signature cyangwa secret muri log |

## Inzira y'umutekano ya API izubakwa

1. `POST /api/wallet/challenges`: server igenzura testnet address, ishyiraho rate limit, ikora nonce ya 256-bit ivuye muri OS RNG, ikabika challenge mbere yo kohereza message umukoresha asinya. Inzira yo guhuza indi wallet na konti isaba session yemewe; login ya mbere ntisaba konti ya kera.
2. `POST /api/wallet/proofs`: server isoma challenge yabitswe, igenzura expiry, network, address na SEP-53 signature. Muri transaction imwe ifunga row ya challenge, igenzura ko itarakoreshejwe, iyishyira `consumed`, ikanashyiraho cyangwa igasoma `wallet_links` na `accounts` ikurikije purpose. Gusubiramo proof birangwa.
3. Session itangwa muri iyo transaction; `GET /api/session` iyisoma, `POST /api/session/logout` iyifunga. Guhindura wallet cyangwa kuyikuraho bizasaba proof nshya n'amategeko ya recovery azasuzumwa. SEP-53 yemeza urufunguzo rwasinye; kuri Stellar multisig ntihita yemeza uburenganzira bwose bwa konti.
4. Wallet proof si payment proof. Buri payment izasinywa na wallet, hanyuma igenzurwe byigenga kuri Stellar mbere ya Receipt Vault.

## Uko bizasuzumwa mbere yo gushyira migration mu bikorwa

- Kwemeza model ya konti, recovery, merchant approval, n'uko wallet nyinshi zahuzwa na konti imwe.
- Gusuzuma SQL migration nshya; kuyigerageza kuri database nshya irimo ubusa, harimo concurrent proof attempts, replay, expiry na uniqueness.
- Kugenzura backup/restore, connection permissions, rate limits, audit na failure recovery.
