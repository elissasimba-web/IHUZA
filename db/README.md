# Web3 testnet database

`web3_testnet/001_identity.sql` yashyizwe kuri PostgreSQL 17 database `ihuza_web3_testnet` iri muri container `ihuza-web3-postgres` ikoresha volume yihariye `ihuza-web3-testnet-pgdata`. `sqlx migrate info --source db/web3_testnet` yemeje ko migration 1 yashyizweho; tables 4 zirahari. Port ifunguye kuri `127.0.0.1:5432` gusa.

Connection yaho ibikwa muri `.env` ya `backend/`, Git irayirengagiza. Ntushyire password muri Git cyangwa mu nyandiko. Kugenzura migration: `sqlx migrate info --source db/web3_testnet`. Mbere yo kongera migrations, banza wemeze `SELECT current_database()` na backup/restore. Nta migration yashyizweho inshuro ebyiri.

Nyuma yo kongera gufungura mudasobwa: `docker start ihuza-web3-postgres`, hanyuma muri `backend/` ukoreshe `sqlx migrate info --source db/web3_testnet`. Niba container isanzwe ikora, nta mpamvu yo kuyirema bundi bushya.

`accounts.qr_reference` ni 32 random bytes zanditswe muri URL-safe base64 nta padding; izatangwa na API nyuma yo kugenzura umukono wa wallet. Ntivuga wallet address. `wallet_links` yemera wallet imwe kuri konti muri iki cyiciro. `wallet_challenges` ibika nonce kugira ngo API izashobore kuyifunga no kuyikoresha rimwe gusa muri transaction. Iyi schema yonyine ntitanga session cyangwa payment.
