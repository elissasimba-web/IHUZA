# IHUZA+ — gahunda ya database nshya ya testnet

Iyi ni gahunda yo gusuzuma mbere yo gushyiraho API cyangwa database ihoraho. `db/web3_testnet/001_identity.sql` yageragejwe muri container y'igihe gito, ariko **ntirakoreshwa kuri database ihoraho**.

## Ahazubakwa

- Database nshya yihariye, urugero `ihuza_web3_testnet`, ifite connection string yayo. Mainnet izakenera database n'amabwiriza byayo ukwabyo.
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

1. `POST /wallet/challenges`: server igenzura testnet address, ishyiraho rate limit, ikora nonce ya 256-bit ivuye muri OS RNG, ikabika challenge mbere yo kohereza message umukoresha asinya. Inzira yo guhuza indi wallet na konti isaba session yemewe; login ya mbere ntisaba konti ya kera.
2. `POST /wallet/proofs`: server isoma challenge yabitswe, igenzura expiry, network, address na SEP-53 signature. Muri transaction imwe ifunga row ya challenge, igenzura ko itarakoreshejwe, iyishyira `consumed`, ikanashyiraho cyangwa igasoma `wallet_links` na `accounts` ikurikije purpose. Gusubiramo proof birangwa.
3. Session itangwa nyuma y'iyo transaction gusa. Guhindura wallet cyangwa kuyikuraho bizasaba proof nshya n'amategeko ya recovery azasuzumwa. SEP-53 yemeza urufunguzo rwasinye; kuri Stellar multisig ntihita yemeza uburenganzira bwose bwa konti.
4. Wallet proof si payment proof. Buri payment izasinywa na wallet, hanyuma igenzurwe byigenga kuri Stellar mbere ya Receipt Vault.

## Uko bizasuzumwa mbere yo gushyira migration mu bikorwa

- Kwemeza model ya konti, recovery, merchant approval, n'uko wallet nyinshi zahuzwa na konti imwe.
- Gusuzuma SQL migration nshya; kuyigerageza kuri database nshya irimo ubusa, harimo concurrent proof attempts, replay, expiry na uniqueness.
- Kugenzura backup/restore, connection permissions, rate limits, audit na failure recovery.
