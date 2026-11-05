# IHUZA+ Web3 branch instructions

- Communicate with the developer mainly in concise Kinyarwanda.
- This branch is the active Stellar implementation. Its localhost wallet proof API uses only the dedicated `ihuza_web3_testnet` database described in `docs/WEB3_DATA_PLAN.md`. New migrations need a reviewed plan.
- The product keeps one persistent opaque IHUZA+ account QR, customer-approved merchant checkout, Smart Invoice, Receipt Vault, and Merchant Assistant. Merchant catalog and prices belong to the merchant.
- A user must control their own Stellar wallet and authorize every asset movement. Never store a private key or seed in the backend, contract, repository, or logs.
- Do not claim any real RWF, CBDC, EBM, bank, MTN, or Airtel integration until implemented, approved, and verified. Testnet value is test value.
- No payment is successful from a submitted hash or server record alone. Verify the chain result, reconcile it, and create durable receipts idempotently.
- Inspect Git status and actual files before edits. Keep changes small, run relevant tests, and report results honestly.
