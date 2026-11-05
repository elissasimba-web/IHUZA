# Web3 testnet database

Migrations `001_identity.sql`, `002_sessions.sql` na `003_merchant_applications.sql` zashyizwe kuri PostgreSQL 17 database `ihuza_web3_testnet` iri muri container `ihuza-web3-postgres` ikoresha volume yihariye `ihuza-web3-testnet-pgdata`. `./scripts/web3_migrate.ps1` yemeza uko zashyizweho. Port ifunguye kuri `127.0.0.1:5432` gusa.

API connection ibikwa muri `.env`, migration owner connection ikabikwa muri `.env.migration`; Git yirengagiza zombi. Ntushyire password muri Git cyangwa mu nyandiko. Kugenzura migration: `./scripts/web3_migrate.ps1`; gukoresha migration nshya: `./scripts/web3_migrate.ps1 -Apply`. Iyi script ikoresha `MIGRATION_DATABASE_URL` ya owner, ikagenzura izina rya database. Mbere yo kongera migrations, banza utegure backup/restore. Nta migration yashyizweho inshuro ebyiri.

API ikoresha role `ihuza_web3_app` ifite SELECT/INSERT kuri `accounts`, `wallet_links` na `security_events`, na SELECT/INSERT/UPDATE kuri `wallet_challenges`. Kuri `account_sessions` ifite SELECT/INSERT na UPDATE ya `revoked_at` gusa; kuri `session_events` na `merchant_applications` ifite SELECT/INSERT gusa. Ntishobora UPDATE `security_events`; ibyo byageragejwe. `.env` ibika `DATABASE_URL` y'iyo role gusa. Ntukoreshe application role mu gukora migrations nshya.

Nyuma yo kongera gufungura mudasobwa: `docker start ihuza-web3-postgres`, hanyuma muri `backend/` ukoreshe `./scripts/web3_migrate.ps1`. Niba container isanzwe ikora, nta mpamvu yo kuyirema bundi bushya.

`accounts.qr_reference` ni 32 random bytes zanditswe muri URL-safe base64 nta padding; itangwa na API nyuma yo kugenzura umukono wa wallet. Ntivuga wallet address. `wallet_links` yemera wallet imwe kuri konti muri iki cyiciro. `wallet_challenges` ibika nonce ikoreshwa rimwe gusa muri transaction. `account_sessions` ibika hash ya token gusa. `merchant_applications` ibika ubusabe butaragenzuwe; nta burenganzira bwa merchant cyangwa payment butangwa n'iyi schema.
