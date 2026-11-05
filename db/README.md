# Web3 testnet database

`web3_testnet/001_identity.sql` ni migration y'ububiko **bushya** `ihuza_web3_testnet`. Ntigomba gukoreshwa kuri database `ihuza` ya kera. Nta data ya kera yimurwa. Yageragejwe muri PostgreSQL 17 container y'igihe gito, yaremye tables 4, kandi yaranze database yitwa `postgres` nta table ishyizeho. Container y'igerageza yasibwe; nta database ihoraho irakoreshwaho iyi migration.

Mbere yo kuyikoresha, banza ukore backup ya Docker volume, wemeze ko uri muri `ihuza_web3_testnet` ukoresheje `SELECT current_database()`, kandi ukoreshe user ufite uburenganzira kuri iyo DB gusa. Migration izakorerwa muri transaction imwe. Ntukoreshe `DROP` kuri database ya kera.

`accounts.qr_reference` ni 32 random bytes zanditswe muri URL-safe base64 nta padding; izatangwa na API nyuma yo kugenzura umukono wa wallet. Ntivuga wallet address. `wallet_links` yemera wallet imwe kuri konti muri iki cyiciro. `wallet_challenges` ibika nonce kugira ngo API izashobore kuyifunga no kuyikoresha rimwe gusa muri transaction. Iyi schema yonyine ntitanga session cyangwa payment.
