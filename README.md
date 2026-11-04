# IHUZA+ Web3 rebuild

This branch starts a new Stellar-based IHUZA+ implementation. The prior Rust API, SQL migrations, and tests remain in Git history and on `main`; the existing PostgreSQL database was not altered.

## Product boundaries

- One persistent, opaque IHUZA+ account QR opens a server-validated person-transfer or approved-merchant checkout context.
- The customer owns and signs with their Stellar wallet. App login or an administrator cannot sign or spend on their behalf.
- Merchant catalog, prices, private Smart Invoices, Receipt Vault, and Merchant Assistant records belong in a privacy-protecting application layer. Any payment claim needs independently verified chain settlement.
- No real RWF token, CBDC, EBM, bank, MTN, or Airtel integration exists in this branch. Testnet assets must be clearly labelled as test value.

## First code slice

`contracts/intent_anchor` is a Rust/Soroban contract that records a single time-limited commitment authorized by its wallet owner. It prevents replay of the same owner/hash while active. It does **not** transfer assets, verify a payment, bind the QR, or issue a receipt. A production intent format must canonically bind network, contract, owner, recipient, asset, amount, invoice reference, nonce, and expiry before this contract could support checkout.

Run `cargo test --locked` from this directory. Build Wasm with `stellar contract build` only after the SDK and CLI compatibility are checked. Do not deploy or use for real funds without an independent security review.

See [the full transition design](docs/WEB3_STELLAR_ARCHITECTURE.md) for the complete customer, merchant, payment, Receipt Vault, and Merchant Assistant roadmap.
