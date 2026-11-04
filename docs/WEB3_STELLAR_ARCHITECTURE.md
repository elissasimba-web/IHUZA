# IHUZA+ — Stellar Web3 architecture transition

Status: whole-platform target architecture and implementation plan, 2026-11-04. This document does not enable real-money operations or claim a deployed Stellar integration.

## Product decision

IHUZA+ will be built as a Stellar-based Web3 application across its customer, merchant, payment, record, and service flows. A customer must authorize any on-chain movement from their own wallet. Soroban is the smart-contract platform; contracts are written in Rust, not Solidity. The existing Rust/Axum/PostgreSQL backend remains the application, privacy, identity, catalog, and indexing layer. Public-chain storage is selected by verifiability and consent requirements, not by the label "Web3".

The existing single persistent opaque account QR stays the entry point. The server resolves it to a role-gated destination and current checkout context. A Smart Invoice or payment request may have a server-side reference, but there is no second invoice/payment QR.

There is no universal rule that an API, database, password login, or merchant dashboard is forbidden in Web3. For IHUZA+, the decisive tests are: who can authorize asset movement, what can be independently verified on Stellar, whether private records stay protected, and whether the app can truthfully reconcile each financial claim. Remove a centralized capability when it violates those tests or a tested replacement makes it redundant, not because its implementation uses PostgreSQL.

## What exists and what does not

- Existing: IHUZA+ identity and sessions, merchant approval and catalog, server-priced pending Smart Invoices with snapshots, one account QR, zero-value RWF wallet accounts, append-only balanced wallet-ledger schema, and draft person/invoice transfer intents.
- Existing read-only Stellar check: `scripts/check_stellar_testnet.ps1` calls RPC `getNetwork` and rejects a response outside the official testnet passphrase. It has no wallet or payment capability.
- Missing: Stellar address binding, proof of wallet ownership, user-side signing, Soroban contracts, Stellar RPC adapter and event indexer, on-chain asset selection, verified on-chain payment state, customer/merchant receipts, and reconciliation.
- The current `wallet_accounts` and `wallet_balances` represent an internal RWF accounting foundation. They are not self-custody Stellar wallets. `DRAFT` intents move no value. No real RWF, CBDC, bank, mobile-money, or EBM integration is operational.

## Root-by-root transition inventory

This table is based on the current routes, Cargo dependencies, and installed SQLx migrations. It is a change order, not permission to erase financial history.

| Existing component | Web3 action | Reason / exit condition |
| --- | --- | --- |
| Rust/Axum API, PostgreSQL, login, roles, audit | Keep and harden | They protect private data and authorize app access; wallet spending additionally requires the user's Stellar signature. |
| One opaque account QR and role-gated resolver | Keep; bind to verified Stellar destination | The QR remains stable while the server validates the current recipient and checkout. No address or secret is embedded in the QR. |
| Merchant catalog, procurement, Smart Invoice snapshots | Keep; connect to verified chain results | Product prices and private purchase details are application data. A pending invoice is not a payment. |
| `POST/GET /wallet` and `wallet_accounts`/`wallet_balances` | Replace in the user experience with a verified Stellar wallet connection and chain-derived asset view; retain existing schema until references and data are audited | The current wallet is a zero-value internal RWF ledger, not a user-controlled chain wallet. Do not present both as one spendable balance. |
| `wallet_transfer_intents` and its draft routes | Evolve into chain-aware intents with network, asset, payer/recipient addresses, expiry, signed payload, and chain result | Existing drafts are server-side preparations with no movement. Migrate by additive schema and verified state transitions. |
| `wallet_ledger_*` and isolated `test_ledger_*` | Keep dormant during transition; evaluate archival or retirement only after the chain accounting and any real RWF obligations are settled | Installed migrations and financial records must not be deleted or rewritten. The runtime role currently cannot post ledger entries. |
| `transactions`, invoice payment status, future receipts | Make verified chain settlement the source of any on-chain payment-success claim; add durable receipt linkage | A submitted transaction hash, notification, or database row cannot prove a completed transfer. |
| Read-only Stellar testnet RPC probe, but no Stellar app/Soroban implementation | Add wallet ownership proof, signing, adapter, event indexer, tests, and only necessary contracts | `scripts/check_stellar_testnet.ps1` verifies `getNetwork` and the testnet passphrase. It does not provide Web3 security or self-custody. |

Remove an old route or table only after its replacement is tested, all callers are moved, existing records are inventoried, and a non-destructive migration path is reviewed. Applied SQL migrations remain immutable. The old RWF ledger and any Stellar asset must have separate names and balance displays throughout the transition.

## Whole-platform domain decisions

The first column describes the actual repository, not a completed Web3 feature. "Off-chain" means the data remains private or operational; it does not mean it is exempt from Web3 security and payment verification.

| Domain | Current repository | Stellar Web3 target and removal rule |
| --- | --- | --- |
| Identity, authentication, roles | Password login, sessions, CUSTOMER/MERCHANT/ADMIN roles, security events | Keep private identity and role checks. Add proof that the logged-in user controls the linked Stellar address; wallet spending requires the user's signature. Remove any assumption that app login can authorize chain spending. |
| Account and QR | One opaque account QR, role-gated resolver, QR SVG | Keep one QR. Resolve it to a current verified wallet destination and approved merchant context. Never make the QR a bearer credential or a source of spend authority. |
| Merchant onboarding | Application and admin approval work | Keep private KYC/business review and audit. If a public merchant attestation is later needed, publish only a minimal revocable proof; merchant approval in PostgreSQL alone cannot grant a contract spending permission. |
| Products and catalog | Merchant-owned RWF prices and product-image references | Keep product details off-chain. Bind a checkout quote to a merchant, item snapshots, currency, expiry, and selected payment asset. The customer reviews any conversion before signing. No contract should set retail prices. |
| Suppliers and procurement | SQL tables exist; no supplier/procurement API in the current routes | Build owner-scoped management and stock movements separately. Keep supplier identities, purchase costs, and private commercial terms off-chain. Only independently authorized settlement references may be linked to chain payments. |
| Smart Invoice | Server-priced pending drafts, item snapshots, owner reads | Keep detailed invoice off-chain. Commit only a minimal opaque reference or privacy-reviewed hash if an on-chain proof is useful. Chain settlement must match the invoice's locked payment terms; a draft remains unpaid. |
| Wallet and person transfer | Zero-value internal RWF wallet and DRAFT intents | Replace user-facing spendable balance with a user-controlled Stellar wallet and independently verified chain asset balance. Retire internal-wallet routes only after a tested replacement; preserve historical financial records. |
| Merchant payment | Invoice-linked DRAFT intent; no payment execution | Bind signed chain operation to recipient, asset, amount, network, invoice, and expiry. Verify success and uniqueness from Stellar before marking paid. A submitted hash is insufficient. |
| Transactions and Receipt Vault | `transactions` table exists; no verified payment or receipt API | Index verified chain results into durable transaction and immutable receipt records for customer and merchant. Keep full receipts private, linked to a minimal chain reference. Refunds and adjustments are new linked records. |
| Notifications and Merchant Assistant | No verified-payment notification or complete sales/profit API | Generate notices and reports from verified payments and traceable stock/cost records. Notifications do not prove goods were delivered, and AI never decides that settlement succeeded. |
| Transport, bills, insurance and healthcare | No current service routes | Add each service after its partner, consent, privacy, fulfillment, and payment contract are specified. Do not place medical, travel, or insurance records on a public chain. |
| External bank/mobile-money/EBM/CBDC | No verified integration | Implement only through documented approved adapters. Keep off-chain settlement and official tax documents distinct from Stellar proof and IHUZA+ receipts. |
| Frontend and deployment | No frontend in this repository | Build accessible wallet confirmation, QR scan, customer checkout, merchant dashboard, and clear testnet labels. Protect signing UI from transaction substitution, malicious scripts, and misleading status. |

## Whole-platform security requirements

1. **Identity and consent:** App sessions identify the user, while wallet signatures authorize chain actions. Address linking requires a one-time, network-specific signed challenge, expiry, and replay protection. Lost-device recovery must be explicit and must not silently transfer custody to IHUZA+.
2. **Data and privacy:** Keep names, phones, item lists, supplier costs, travel/health records, and full receipts off-chain. Review even hashes for correlation or dictionary attacks before publishing. Encrypt sensitive data where appropriate and retain auditable access rules.
3. **QR and checkout:** Resolve the account QR on the server, check current merchant approval and wallet binding, lock quote terms, and show the payer the final destination, asset, amount, network, and fee before signing. Recheck those values in the chain result.
4. **Contract and asset controls:** Define each Soroban contract's exact responsibility and its admin, upgrade, pause, recovery, authorization, and event rules. Test wrong signer, forged destination, replay, duplicate payment, reentrancy where applicable, storage expiry, and unauthorized upgrade. A token issuer's freeze or clawback powers must be documented honestly.
5. **Settlement and records:** Distinguish prepared, submitted, failed, and verified operations. Index chain events idempotently; tolerate RPC failure and restart without double receipts. Reconcile on-chain value, internal records, and any regulated off-chain reserve or provider settlement.
6. **Operations:** Separate development, testnet, and production credentials. Never put user signing secrets in server env vars, logs, or Git. Review dependencies, build artifacts, deployment keys, RPC trust, rate limits, monitoring, incident response, backups, and independent security findings before real funds.

No current test or testnet RPC probe proves these requirements have been met. The existing Rust tests cover selected API and database behavior; they do not audit Soroban, wallet custody, frontend signing, or end-to-end settlement.

## Ownership and custody

- Target: the user controls the signing credential for their Stellar wallet. The backend stores only a public wallet address and a verified ownership binding. Login to IHUZA+ alone cannot authorize spending.
- A transfer must display network, asset, recipient, amount, and fee to the user before signing. A server cannot replace the recipient or amount after the user approves.
- Begin with an established non-custodial Stellar wallet connection on testnet. Evaluate a passkey-based smart wallet for accessible onboarding only after its recovery and signer policies are specified and tested. Recovery must not silently give IHUZA+ unilateral spending authority.
- Never store a user's private key, seed phrase, passkey secret, or unencrypted signing material in PostgreSQL, `.env`, logs, or Git. Do not derive a chain key from the IHUZA+ login password.
- Document how lost devices, stolen credentials, wrong recipients, account suspension, and disputes work. A completed chain transfer cannot be assumed reversible.

## Asset and settlement boundary

- First working demo: Stellar testnet assets only, clearly labelled test value. No test token is represented as RWF, customer deposits, or a legally settled merchant payment.
- The production asset representing spendable RWF is undecided. An internal RWF ledger claim, a regulated Stellar-issued asset, and a future CBDC are different instruments with different issuer, custody, redemption, reserve, and approval requirements.
- Before any real customer value is accepted, document the legal issuer or licensed service, safeguarding and reserve reconciliation, approved funding and withdrawal routes, asset controls, and the regulatory basis for merchant payments. A technical partnership or winning a competition does not itself provide authorization.
- No IHUZA+ cryptocurrency or stablecoin is issued in this transition phase.

## Payment flow

1. Customer scans the merchant's one IHUZA+ account QR. Backend verifies merchant role and serves the merchant-owned catalog.
2. Backend calculates a quote from current prices and records an immutable pending Smart Invoice after customer approval.
3. Backend prepares a payment intent with immutable destination, asset, amount, network, invoice reference, and expiry. Client verifies those fields and the user signs using their own wallet.
4. Backend submits or observes the signed operation and independently verifies the Stellar network, successful result, asset, source, destination, amount, and unique transaction/event reference. Submission alone is never success.
5. Only after verified settlement does the system link the payment to the invoice, create an ordinary IHUZA+ receipt for both parties in Receipt Vault, and send a neutral notification. Receipt creation and index updates must be idempotent and recoverable.
6. Refunds and adjustments are separate authorized events linked to the original receipt. They do not silently edit the original invoice or automatically debit a customer.

For person-to-person QR scans, present a person transfer flow without a merchant catalog. The authenticated payer still signs. No scan alone transfers value.

## On-chain and off-chain responsibilities

| Stellar / Soroban | IHUZA+ backend / PostgreSQL |
| --- | --- |
| Asset ownership, user authorization, transaction result, minimal payment reference or event where justified | Identity, roles, merchant catalog, prices, detailed invoices and receipts, private information, notifications, audit and searchable indexes |

Use Stellar native payment and asset features where they satisfy the requirement; add a Soroban contract only for behavior that requires programmable authorization or payment rules. A first contract proposal must define its exact responsibility and avoid duplicating native asset transfer. Store no names, phone numbers, item lists, or full invoice contents on-chain. A hash or opaque payment reference can link a chain event to a private invoice, with a versioned canonical encoding and privacy review.

## Security and accounting gates

- Contract authorization must bind the actual wallet owner, network, asset, recipient, amount, intent, and expiry. Test replay, duplicate submission, malicious substitution, overspending, cancellation, and concurrent attempts.
- Review contract upgrade/admin/pause powers so IHUZA+ cannot use them to bypass user consent. Independent security review is required before any mainnet contract handles real customer value.
- Run an idempotent event indexer. Reconcile chain observations with payment intents, internal accounting, and any external funding or safeguarded reserves. A database row alone cannot prove on-chain settlement.
- Preserve the existing internal ledger as historical/dormant accounting infrastructure while the asset and reconciliation model is designed. Do not mutate applied migrations or relabel its balance as a Stellar balance. Any schema extension uses additive migrations.
- Merchant Assistant, invoices, and Receipt Vault continue as product services. Their financial claims must be derived from verified payments; tax/EBM records remain separate until an official integration exists.

## Implementation order

1. Inventory every current route, table, test, and data dependency against the whole-platform decisions above. Mark what is retained, replaced, or retired, with a reversible path for each replacement.
2. Define the testnet asset, non-custodial wallet/recovery model, merchant address binding, quote-to-chain conversion rules, contract scope, and privacy boundaries. Keep production RWF status unresolved until issuer and redemption details are known.
3. Implement user-owned Stellar address binding with a signed, expiring, network-specific challenge. Extend identity and merchant approval without granting app administrators chain spending powers.
4. Build the signed testnet person-transfer and one-QR merchant-checkout flows, using current product and invoice snapshots. Add a minimal Soroban contract only for behavior native Stellar payments cannot provide.
5. Verify chain results independently and index them idempotently. Build immutable Receipt Vault records, merchant notifications, disputes, and linked refunds; connect Merchant Assistant sales and accounting to those verified records.
6. Implement the customer and merchant frontend, owner-scoped supplier/procurement/stock flows, and any additional services only after their privacy and partner rules are defined. Remove superseded draft-only routes after migration tests pass.
7. Run contract, API, frontend, concurrency, privacy, key-recovery, and end-to-end security reviews. Demonstrate only clearly labelled testnet value in competitions.
8. Complete issuer, licensing, safeguarding, partner, EBM/tax, and reconciliation work before considering real-money or mainnet activation. Retire legacy tables only after historical and legal obligations are satisfied.

No production network, live RWF asset, on-chain contract, or real-money route is selected or enabled by this document.

## Technical references

- Stellar account authorization: https://developers.stellar.org/docs/data/apis/horizon/api-reference/resources/accounts
- Soroban Rust SDK and supported build target: https://github.com/stellar/rs-soroban-sdk
- Stellar security guidance for web applications: https://developers.stellar.org/docs/build/security-docs/securing-web-based-projects
- Stellar RPC `getNetwork`: https://developers.stellar.org/docs/data/apis/rpc/api-reference/methods/getNetwork
