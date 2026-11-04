# IHUZA+ — Stellar Web3 architecture transition

Status: target architecture and implementation plan, 2026-11-04. This document does not enable real-money operations or claim a deployed Stellar integration.

## Product decision

IHUZA+ will be built as a Stellar-based Web3 application. A customer must authorize any on-chain movement from their own wallet. Soroban is the smart-contract platform; contracts are written in Rust, not Solidity. The existing Rust/Axum/PostgreSQL backend remains the application, privacy, identity, catalog, and indexing layer. "Web3-first" does not require putting private customer or invoice data on a public ledger.

The existing single persistent opaque account QR stays the entry point. The server resolves it to a role-gated destination and current checkout context. A Smart Invoice or payment request may have a server-side reference, but there is no second invoice/payment QR.

## What exists and what does not

- Existing: IHUZA+ identity and sessions, merchant approval and catalog, server-priced pending Smart Invoices with snapshots, one account QR, zero-value RWF wallet accounts, append-only balanced wallet-ledger schema, and draft person/invoice transfer intents.
- Missing: Stellar address binding, proof of wallet ownership, user-side signing, Soroban contracts, Stellar RPC adapter and event indexer, on-chain asset selection, verified on-chain payment state, customer/merchant receipts, and reconciliation.
- The current `wallet_accounts` and `wallet_balances` represent an internal RWF accounting foundation. They are not self-custody Stellar wallets. `DRAFT` intents move no value. No real RWF, CBDC, bank, mobile-money, or EBM integration is operational.

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

1. Agree the wallet custody/recovery model and first testnet asset, and map the existing internal RWF ledger to the intended Stellar asset model.
2. Add user-owned Stellar address binding with signed challenge, network separation, replay protection, and owner-scoped reads. No private keys on the server.
3. Add a read-only Stellar testnet adapter and verification tests; then a signed testnet transfer flow with explicit user confirmation.
4. Specify and test the minimal Soroban contract only after deciding what native Stellar payments cannot do. Add event indexing and restart/replay recovery tests.
5. Link verified testnet payment events to the one-QR checkout, pending Smart Invoice, Receipt Vault, and merchant notifications. Build a demonstrable customer and merchant UI.
6. Run end-to-end, concurrency, abuse, custody-recovery, and independent contract/security reviews. Prepare a clear competition demo that labels all test funds.
7. Complete the issuer, licensing, safeguarding, partner, privacy, and reconciliation work before considering real-money or mainnet activation.

No production network, live RWF asset, on-chain contract, or real-money route is selected or enabled by this document.
