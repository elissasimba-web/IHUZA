# IHUZA+ catalog ya testnet

Iyi ni gahunda yasuzumwe mbere ya migration `005_merchant_products.sql`. Catalog iba off-chain, igiciro kigashyirwaho na nyiri merchant account. Kuri iyi prototype, `RWF` ni **ikimenyetso cy'igiciro mu bicuruzwa gusa**; nta RWF asset, wallet balance cyangwa payment bivuga.

## Intambwe ya mbere

- Ongeraho `merchant_products` muri `ihuza_web3_testnet` gusa: product UUID, merchant account, izina, `unit_price` ya decimal ifite digits 2, `currency_code = RWF`, n'igihe byandikiwe. Nta wallet address, seed cyangwa private data bijyamo.
- API yemere merchant wemejwe na votes 2 muri reviewer set iriho gusa gukora product. Ikura account ID muri session, ntizamuka kuri client. Name n'igiciro byagenzurwa kuri server; nta total cyangwa invoice ikomoka kuri client.
- Public catalog isomwa hakoreshejwe **QR reference imwe isanzwe** kandi gusa merchant approval ikiriho. Iyo approval ibuze cyangwa reviewer set ihindutse, catalog ifungwa. QR ubwayo ntihinduka.
- Ubu ni `create` na `list` gusa. Guhindura ibiciro bizakorwa n'amateka y'ibiciro asobanutse kugira ngo Smart Invoice izabike snapshot y'igiciro yakoreshejwe. Nta checkout cyangwa payment muri iki gice.
- Public list y'iyi prototype yerekana ibicuruzwa 100 bya mbere kuri konti; pagination izakenerwa mbere y'uko catalog iba nini. Nta price edit cyangwa deletion byubatswe.

## Isuzuma rya migration

- Migration nshya ifite database-name guard, FK kuri merchant application, positive price check, na SELECT/INSERT gusa kuri runtime role. Nta table ya kera ihindurwa, nta DELETE, nta wallet balance.
- Mbere yo kuyikoresha: `git diff --check`, SQL review, kugenzura ko `ihuza-web3-postgres` na `ihuza_web3_testnet` ari byo. Nyuma yayo: migration info, ibizamini by'uburenganzira (pending/approved/other account), validation y'igiciro, public read na reviewer set rotation.
