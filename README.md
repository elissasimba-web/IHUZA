# IHUZA+

**One Account. Every Service.**

IHUZA+ ni umushinga wa serivisi z'imari n'iz'ubucuruzi ugenewe u Rwanda. Ubu iyi repository ni **prototype ya Stellar Web3**; si app yuzuye cyangwa serivisi yakira amafaranga nyayo.

## Ibyakozwe kandi byagenzuwe

- Rust workspace irimo Soroban `intent_anchor` contract. Nyiri aderesi ya Stellar agomba kwemeza intent hash; contract igenzura igihe ntarengwa kandi ikumira gusubiramo iyo hash ikiri mu bubiko. Nta mafaranga iyi contract yimura.
- `cargo test --locked --offline` yatsinze; `stellar contract build` yatanze Wasm. Nta contract yashyizwe kuri testnet cyangwa mainnet.
- `scripts/check_stellar_testnet.ps1` ni read-only network probe; ntabwo ari payment integration.
- `web/` ifite frontend ihuza Freighter wallet kandi ikemeza ko iri kuri Stellar testnet. Yerekana public address gusa; nta transaction ikora kandi nta konti ya IHUZA+ ihuzwa na wallet kugeza ubu.

## Ibyo project izubaka

Umukiriya azakoresha wallet yigenzurira, ascan-e **QR imwe ihoraho** ya konti y'umucuruzi, abone catalog n'ibiciro byashyizweho n'uwo mucuruzi, ahitemo ibicuruzwa, abone Smart Invoice yagenzuwe na server, hanyuma yemeze transaction muri wallet ye. Nyuma y'uko ubwishyu bwemejwe byigenga kuri Stellar, IHUZA+ izabika inyemezabwishyu muri Receipt Vault no kumenyesha umucuruzi. QR ya konti isanzwe izafungura uburyo bwo kohereza amafaranga hagati y'abantu, nta catalog y'umucuruzi.

Merchant Assistant izafasha umucuruzi kwandika suppliers, procurement, stock movements, ibicuruzwa, sales na reports. Ibyo byose n'inyandiko z'abakiriya bizabikwa mu buryo burinda amakuru yihariye; si ngombwa kubishyira kuri blockchain.

Reba [ibyo twemeranyijwe ku bicuruzwa](docs/PRODUCT_REQUIREMENTS.md) n'[imyubakire ya tekiniki](docs/ARCHITECTURE.md). Iyi prototype ntikoresha database cyangwa API kugeza ubu.

## Gutangira

Ibisabwa: Rust, `wasm32v1-none` target, na Stellar CLI ihuye na Soroban SDK v26.

```powershell
cargo test --locked
stellar contract build
cd web
npm ci
npm test
npm run dev
```

Fungura `http://127.0.0.1:5173` muri browser ifite Freighter extension, uyishyire kuri testnet, hanyuma ukande **Huza Freighter**. Kuri browser itagira Freighter, UI yerekana icyo kubura; ntihimba wallet. `npm run build` igenzura production bundle.

Nta mabanga ya wallet ashyirwa muri iyi repository. Nta RWF token, CBDC, EBM, bank, MTN cyangwa Airtel integration irimo. Iyimurika rigomba kwerekana prototype nk'uko iri, rikavuga neza ko ubwishyu nyabwo na Receipt Vault bitarubakwa.
