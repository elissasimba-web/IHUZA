# IHUZA+ payment and ledger design — review draft

Status: test-only ledger prototype installed on 2026-11-03. The intended product has an IHUZA+ wallet for holding value in each user's account and making internal person-to-person and merchant payments. No production wallet ledger, funding rail, settlement, or verified receipt is implemented.

## Agreed wallet direction (2026-11-03)

- IHUZA+ owns the customer-facing wallet, account experience, internal payment flow, and ledger design. A customer can retain wallet value and send it to another IHUZA+ account. An external provider must not be hardcoded as the wallet or required for each internal transfer.
- Bank and mobile-money connections are future, separately verified funding, withdrawal, or interoperability routes. An IHUZA+ wallet balance must never be created from an unverified external request or a client-supplied success flag.
- A wallet balance represents a claim on safeguarded real value, not money created by inserting a database row. The legal issuer, license, safeguarding institution or arrangement, reserve accounting, and settlement processes must be confirmed before real customer funds are accepted. IHUZA+ owning the wallet experience does not itself establish authorization to hold customer funds.
- Rwanda CBDC/e-FRW is a possible future integration only if officially issued, accessible, approved, and tested. The wallet and internal ledger must work independently of that possibility.
- The initial wallet design is RWF and online-only. Other assets or currencies require separate approval and accounting rules.

## What exists now

- Customer checkout creates a server-priced `PENDING` Smart Invoice with item snapshots and an idempotency key.
- Invoice list and detail are scoped to the customer and merchant owner.
- The applied `transactions` migration has amount, currency, payment method/reference, and status, but no API writes to it yet.
- `test_ledger_*` tables and `test_ledger_transfer` provide an isolated RWF test ledger with paired postings, a posted-balance view, key-based retries, and overspend checks for test users. No API invokes it; no production ledger exists.
- There is no funding rail, provider callback verification, reconciliation process, or Receipt Vault record of a verified payment.

## Gaps in the applied schema

- `transactions.payment_reference` is nullable and is not an idempotency key for a customer's request. It cannot prevent two payment attempts for the same checkout.
- Nothing in the current schema requires a `SUCCESS` transaction to have verified funding or balanced ledger postings.
- `invoices.payment_status` has only `PENDING`, `PAID`, `CANCELLED`, and `REFUNDED`; it cannot accurately represent partial payments, advances, or debt repayment by itself.
- `invoices.qr_reference` comes from an applied migration. It must remain unused for a separate invoice QR; the one persistent account QR remains the checkout entry point.
- An IHUZA+ payment record is not an official EBM invoice.

## Proposed separation of responsibilities

1. **Payment intent:** An authenticated payer explicitly requests an internal transfer or payment for a specific invoice and amount. The server verifies ownership, currency, invoice state when applicable, and amount. A unique key per payer plus a request hash makes retries return the same intent; reuse with different data is rejected.
2. **IHUZA+ wallet:** Each eligible user has an internal RWF wallet account. Available value comes only from verified funding or prior balanced internal postings. An internal transfer debits one wallet and credits another in one database transaction, with sufficient-funds, authorization, idempotency, and concurrency controls.
3. **External rail adapter:** A provider-specific module handles future funding, withdrawal, or interoperability. Only configured, approved, tested rails may be enabled. Internal IHUZA+ transfers do not require a provider call for every transfer.
4. **Verification:** An external provider callback or status response is authenticated, matched to the intent, and checked for amount, currency, provider reference, and allowed state transition. Client input and a notification cannot mark external funding successful.
5. **Ledger:** A successful value movement posts balanced, append-only entries in the same database transaction as the final internal payment state. Each posting batch has one source event and currency; debits and credits must sum to zero. Balances are derived from postings or a controlled projection, never maintained as the sole record.
6. **Invoice and receipt:** The invoice records what was charged; payment records what was transferred. A verified successful payment creates a linked IHUZA+ receipt snapshot for customer and merchant. It does not by itself confirm goods were delivered. Refunds and adjustments create linked records rather than editing the original receipt.
7. **Reconciliation:** The sum of customer wallet liabilities must be reconciled with safeguarded external value, and provider or bank records must be matched against funding/withdrawal attempts and ledger batches. Missing or conflicting entries become exceptions for review; the system does not silently repair them.

## Required database design before a payment API

- Add payment-intent idempotency and request-hash fields, explicit rail and verification metadata, provider event uniqueness, and allowed state transitions through new corrective migrations. Keep applied migration files unchanged.
- Add ledger accounts, posting batches, and postings with immutable history, currency, source references, and database constraints. A single transaction must lock the intent and affected ledger accounts, verify sufficient available funds where applicable, post both sides, and commit once.
- Model partial payment, advance, debt, cancellation, and refund separately from fulfillment. Do not infer `PAID` from one arbitrary transaction row.
- Add a durable receipt record tied to verified payment and invoice snapshots; distinguish it from any future officially integrated EBM document.

## Test and release gates

- Duplicate requests and duplicate/out-of-order provider callbacks must have one financial effect.
- Concurrent payments from the same funding source must not overspend.
- Failure after any step must roll back state and postings together or leave an explicit recoverable pending state.
- Ledger batches must balance; negative or mismatched amounts/currencies must be rejected.
- Reconciliation must detect missing, duplicate, and differing provider records.
- Customer and merchant authorization, refunds, and linked receipt history need database integration tests.
- No real-money route may be enabled until a lawful funding/settlement rail, credentials, callback verification, and operational reconciliation have been agreed and tested.

## Decisions needed before enabling real funds

1. Confirm the legal issuer/licensing path and how customer funds will be safeguarded and reconciled. This may involve an external safeguarding institution without making the IHUZA+ wallet provider-owned.
2. Select and verify at least one way to fund and withdraw real RWF; an internal transfer cannot create its own initial value.
3. Decide whether the first release supports full invoice payment only, with partial payments, advances, and debt as later explicit flows.
4. Agree fee and refund rules. No fee or tax amount should be invented by the backend.

The test ledger proves basic posting rules, but it is deliberately separate from invoices and `transactions`. Its database integration test covers balanced entries, idempotent retry, insufficient test funds, append-only entries, and rollback. Concurrent transfer safety and any real-money flow remain unverified. The production wallet ledger schema and internal-transfer service can be built and tested without choosing a specific external provider; accepting real customer funds must wait for the decisions and verification above.
