# IHUZA+ payment and ledger design — review draft

Status: test-only ledger prototype installed on 2026-11-03; production payment design remains for review. No payment rail, wallet balance, settlement, or verified receipt is implemented.

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

1. **Payment intent:** An authenticated payer explicitly requests payment for a specific invoice and amount. The server verifies ownership, currency, invoice state, and amount. A unique key per payer plus a request hash makes retries return the same intent; reuse with different data is rejected.
2. **Payment rail adapter:** A provider-specific module initiates or checks a payment. Only configured, approved, tested rails may be enabled. The core flow does not assume MTN, Airtel, a bank, CBDC, or any provider is already connected.
3. **Verification:** A provider callback or status response is authenticated, matched to the intent, and checked for amount, currency, provider reference, and allowed state transition. Client input and a notification cannot mark payment successful.
4. **Ledger:** A successful value movement posts balanced, append-only entries in the same database transaction as the final internal payment state. Each posting batch has one source event and currency; debits and credits must sum to zero. Balances are derived from postings or a controlled projection, never maintained as the sole record.
5. **Invoice and receipt:** The invoice records what was charged; payment records what was transferred. A verified successful payment creates a linked IHUZA+ receipt snapshot for customer and merchant. It does not by itself confirm goods were delivered. Refunds and adjustments create linked records rather than editing the original receipt.
6. **Reconciliation:** Provider or bank statements are matched against payment attempts, provider references, amounts, currency, and ledger batches. Missing or conflicting entries become exceptions for review; the system does not silently repair them.

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

## Decisions needed before implementing production ledger/payment migrations

1. Which first rail will supply and settle real funds, and who holds customer funds? A provider integration cannot be assumed from the existing code.
2. Should the first release support full invoice payment only, with partial payments, advances, and debt kept as later explicit flows?
3. What fee and refund rules apply? No fee or tax amount should be invented by the backend.

The test ledger proves basic posting rules, but it is deliberately separate from invoices and `transactions`. Its database integration test covers balanced entries, idempotent retry, insufficient test funds, append-only entries, and rollback. Concurrent transfer safety and any real-money flow remain unverified. The next production work must wait for the funding and settlement decisions above.
