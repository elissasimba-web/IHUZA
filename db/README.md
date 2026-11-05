# Web3 testnet database

`web3_testnet/001_identity.sql` ni migration y'ububiko **bushya** `ihuza_web3_testnet`. Yageragejwe muri PostgreSQL 17 container y'igihe gito, yaremye tables 4, kandi yaranze database yitwa `postgres` nta table ishyizeho. Container y'igerageza yasibwe; nta database ihoraho irakoreshwaho iyi migration.

Mbere yo kuyikoresha, banza wemeze ko uri muri `ihuza_web3_testnet` ukoresheje `SELECT current_database()`, kandi ukoreshe user ufite uburenganzira kuri iyo DB gusa. Migration izakorerwa muri transaction imwe. Tegura backup/restore ku bubiko bushya mbere y'ibikorwa by'amafaranga nyayo.

`accounts.qr_reference` ni 32 random bytes zanditswe muri URL-safe base64 nta padding; izatangwa na API nyuma yo kugenzura umukono wa wallet. Ntivuga wallet address. `wallet_links` yemera wallet imwe kuri konti muri iki cyiciro. `wallet_challenges` ibika nonce kugira ngo API izashobore kuyifunga no kuyikoresha rimwe gusa muri transaction. Iyi schema yonyine ntitanga session cyangwa payment.
