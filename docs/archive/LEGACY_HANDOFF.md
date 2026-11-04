# Historical handoff (archived)

This is the old project context, kept for history. It is not current instructions. Local database credentials were redacted. See ../PRODUCT_REQUIREMENTS.md and ../ARCHITECTURE.md for the active direction.



You are now working on the IHUZA+ project.

CURRENT DIRECTION (2026-11-04): The developer has chosen a Stellar Web3-first architecture for the entire IHUZA+ platform, with Soroban smart contracts where needed and user-controlled wallet signing. Review `backend/docs/WEB3_STELLAR_ARCHITECTURE.md` before extending any domain, including identity, QR, merchant tools, invoices, payments, receipts, notifications, and future services. The earlier internal RWF-wallet design below records historical work and remains dormant; it is not a self-custody Stellar wallet. Preserve the single account QR, private merchant/invoice data, audit history, and installed migrations while replacing outdated flows in tested stages. Do not claim a live Stellar asset, on-chain payment, or real-money authorization exists. No production asset or custody/recovery model has been finalized. A Web3 platform may use an off-chain backend for private and operational data; customer money movements require user authorization and independently verified settlement.

Before making any changes, read the entire AGENTS.md file in the project root.

IMPORTANT RULES:

    Do not restart, recreate, or scaffold the project. Continue from its existing state.

    Inspect the actual repository files, Git status, Cargo dependencies, SQL migrations, and available tests before deciding what to do.

    Compare the actual code with the project context in AGENTS.md. If something differs, report the difference rather than assuming the documentation is correct.

    The create_suppliers migration was previously pending. Verify current migration history before taking action; it was installed when checked on 2026-11-02.

    PostgreSQL uses Docker container ihuza-postgres, database ihuza, and local development port 5432. Check the actual connection state before diagnosing connection failures.

    Never delete or rewrite applied database migrations casually. Use corrective migrations when needed.

    Preserve the agreed IHUZA+ architecture, especially the unified IHUZA+ QR, payment abstraction layer, Smart Invoice, Receipt Vault, and Merchant Assistant.

    Do not claim that bank, MTN, Airtel, CBDC, EBM, or other integrations exist unless they have actually been implemented and verified.

    Protect financial integrity. Do not implement a wallet as a simple mutable balance without a proper ledger, database transactions, idempotency, and reconciliation design.

    Explain your work to the developer mainly in Kinyarwanda, using short, practical steps.

    Do not make broad architectural changes or destructive database changes without explaining the reason and obtaining approval.

YOUR FIRST TASK:

Inspect the current project and report:

    What is confirmed complete.

    What remains incomplete.

    Whether the create_suppliers migration is pending or applied.

    Whether the database connection works.

    The safest exact next step.

Do not implement unrelated features yet. Start by verifying the actual state of the repository and database.
IHUZA+ — Master Project Context for Codex
1. Project identity

    Project: IHUZA+
    Company: IHUZA Digital ELI Technologies Ltd
    Main slogan: One Account. Every Service.
    Receipt Vault slogan: Everything You Pay, Everything You Keep.
    Initial market: Rwanda
    Primary communication with the developer: Kinyarwanda. Keep explanations short and practical; English technical terms/code are fine.

2. How to work with the developer

Use this loop:

    Briefly explain the next action in Kinyarwanda.
    Give the exact command or make a small, well-scoped code change.
    Run/check the relevant result where tools allow.
    Report what actually happened; never assume a command succeeded without evidence.
    Continue with the next step.

Avoid long theory. Avoid repeatedly asking the developer to paste whole files when a success/error output is enough. Ask for targeted output only when needed to diagnose a real issue.

Before a major feature:

    Check it against the IHUZA+ vision and existing architecture.
    Explain any conflict before changing direction.
    Prefer small, tested changes over large speculative rewrites.
    Do not restart the project or repeat completed setup.
    Do not overwrite existing work without inspecting it first.
    If working in a local repository through Codex, inspect the real files and Git status before making changes.

3. Product vision

IHUZA+ is intended to be a Rwanda-focused digital wallet and multi-service platform—not merely another payment app. Core idea: one account connects payments, services, records, and users.

Long-term services may include:

    Digital payments and wallet
    Merchant payments and merchant tools
    Transport/bus payments
    Bill payments
    Insurance and healthcare-related payments/services
    Product and inventory management
    Payment notifications and automation
    Financial records and Receipt Vault
    Integration with banks, mobile-money providers, and other approved payment rails
    Future Rwanda CBDC integration only if/when officially available and approved
    Potential USDT/other digital-asset rails only where legally and technically appropriate

Agreed wallet direction (2026-11-03): IHUZA+ is intended to provide its own customer-facing RWF wallet. Users should be able to retain value and transfer it between IHUZA+ accounts, including customer-to-customer and customer-to-merchant payments. External banks/mobile-money providers are future funding, withdrawal, or interoperability partners, not owners of the IHUZA+ wallet or mandatory hops for each internal transfer. A wallet credit requires verified real funding or a balanced internal transfer; a database entry must never create unbacked customer value. Before accepting real funds, establish the applicable license/issuer, safeguarding arrangement, reserve reconciliation, and operational controls. Future CBDC integration remains conditional on official issuance, approval, and testing.

The platform should reduce unnecessary physical-cash handling, queues, confusion, and payment disputes in Rwanda, including at busy bus stations. Design for accessibility, reliability, and ordinary users.
4. IHUZA+ QR and payment architecture — important agreed direction

Each IHUZA+ account has one persistent unified IHUZA+ QR. The same account QR is used for paying or receiving according to the authenticated users, roles, and server-side checkout context. It is not locked to one provider such as MTN, Airtel, or one bank.

Agreed product concept:

    The customer scans the merchant's single IHUZA+ account QR to open that merchant's server-validated checkout context and catalog.
    In the agreed initial checkout flow, the customer scans the merchant's QR. A merchant must not scan a regular customer's QR to create a sale, send goods, or treat that customer as a merchant.
    A regular customer account must not expose a merchant catalog or merchant-order flow unless that user has separately registered and been authorized as a merchant.
    Any different future use of a customer's account QR must be discussed, approved, role-gated, and documented before implementation.
    Do not create a separate merchant QR and invoice QR. A Smart Invoice is a server-side record linked to the participating accounts and checkout/payment references; it does not require a second QR.
    The customer should not have to choose a separate provider before using the IHUZA+ flow.
    The customer needs an IHUZA+ account for the agreed core flow.
    If the customer's IHUZA+ wallet has insufficient funds, the product may offer a supported funding/payment route from a bank or other integrated institution, subject to actual integrations, consent, and regulation.
    Payment orchestration chooses among supported and eligible payment rails according to available funds, configuration, and rules.
    The architecture must be modular so future approved Rwanda CBDC support can be added without rewriting the core payment model.
    Do not claim integrations already exist. QR generation, orchestration, external funding, CBDC, MTN/Airtel, and bank connections must be treated as future work until implemented and verified. Offline payments are out of scope unless the developer explicitly reopens that decision in the future.
    Keep payment/asset abstraction separate from core transaction logic. Never hardcode the system to one payment provider.
    The account QR must use an opaque, non-guessable account reference. It must never contain secrets, balances, prices, invoice details, or sensitive customer data. The server determines the account, role, merchant catalog, and current checkout/payment context.

5. Smart Invoice, Transaction, Receipt Vault — agreed semantics

These are related but distinct concepts:
Smart Invoice

Records what is being charged for:

    Merchant
    Optional identifiable customer
    Invoice number/reference
    Product/service lines
    Quantity
    Unit price
    Line subtotal and total
    Currency
    Date/time
    Payment status
    Account, checkout, and payment references as appropriate

The merchant should be able to add products and set their own prices. Different merchants may sell the same product at different prices.

Invoice items must preserve historical values (product name and unit price at time of sale) so later product edits do not change old invoices. Totals must be validated/calculated safely on the server, not trusted from client input.
Transaction / payment

Records what a payment attempt or completed payment did:

    Sender/payer when known
    Receiver/payee
    Amount and currency
    Payment method/rail
    Provider reference when applicable
    State and timestamps
    Related invoice when applicable
    Idempotency/reference data
    Refund/failure/cancellation information as appropriate

Payment status and invoice status must be modeled carefully. A payment can be partial, an advance, a debt repayment, a refund, or another arrangement. Do not assume every payment means goods were delivered.
Receipt Vault

Customers and merchants should be able to find the relevant record later:

    Merchant and customer when identifiable
    Products/services and quantities
    Amount and currency
    Date/time
    Transaction/reference ID
    Payment method
    Relevant tax information where applicable

IHUZA+ can store its own transaction/receipt records, but must not claim those records are official EBM invoices or that IHUZA+ replaces Rwanda's EBM system. EBM/tax compliance requires official legal and technical approval/integration. Build a future integration boundary without pretending approval exists.

5A. QR checkout, item reconciliation, and receipt creation

The agreed customer checkout flow is:

    The customer scans the merchant's single unified IHUZA+ account QR.
    That account QR opens the merchant's server-validated catalog so the customer can select products and quantities.
    The server creates or updates the Smart Invoice from the approved checkout data. There is no separate invoice QR.
    A customer who wants goods sent remotely should use a future chat/order-request flow rather than having the merchant scan the customer's QR.
    Do not implement chat or remote ordering until messaging, consent, merchant acceptance, item and price confirmation, delivery, cancellation, refund, dispute, privacy, blocking/reporting, record-retention, and applicable legal rules have been agreed.
    Chat must not create a payment, invoice, delivery obligation, or automatic debit by itself. A merchant offer/order proposal and the customer's explicit approval must precede any payable Smart Invoice.
    The merchant owns the catalog and sets the selling prices. IHUZA+ does not set the merchant's retail prices.
    The server must recalculate and validate all item prices and totals; never trust totals supplied by the client.
    The customer reviews and approves the selected items and server-calculated total, then may pay without waiting for the merchant to review the list.
    A verified successful payment notifies the merchant of the amount and items paid for; a notification must not be treated as proof that goods were provided.
    If the merchant notices a possibly omitted item, the merchant must first check what was actually provided before making a claim.
    The merchant may propose a separate adjustment showing the merchant name, items already paid for, the possibly omitted item, quantity, price, and additional amount. The customer may approve or dispute it.
    A merchant adjustment must never silently change the original payable total or trigger an automatic debit.
    After a verified successful payment, IHUZA+ creates an ordinary IHUZA+ invoice/receipt immediately and stores it in Receipt Vault for both customer and merchant.
    The receipt preserves immutable item, quantity, price, total, currency, merchant, customer when known, transaction reference, payment method, status, and date/time snapshots.
    If an omitted item is discovered after payment, do not edit the original receipt. Create a separate payment-adjustment request or new invoice that the customer may approve or dispute.
    A disputed adjustment must not debit the customer automatically. Keep the original receipt and the dispute/adjustment history.
    Notifications should neutrally show the merchant, items already paid, the proposed omitted item, and any additional amount. Do not accuse either party or decide who is at fault automatically.
    Merchant Assistant or AI may summarize verified records and help identify a possible mismatch, but it must not decide that payment succeeded, alter totals, debit a customer, or confirm fulfillment.

Receipt Vault is the durable customer-and-merchant record for all verified IHUZA+ receipts. Refunds, cancellations, and adjustments must be represented as linked states or records rather than erasing or silently rewriting history.

When an official EBM integration is later approved, implemented, and verified, Receipt Vault may store the IHUZA+ presentation together with the official EBM reference/document and required tax information. Until then, IHUZA+ receipts must not be represented as official EBM invoices.

6. Merchant Assistant — agreed Phase 5 scope

The agreed list is:

PHASE 5 — MERCHANT ASSISTANT

    Merchant registration
    Suppliers
    Procurement
    Products
    Stock
    Purchase cost
    Sales
    Revenue
    Profit
    Reports
    Tax-related information

Merchant Assistant is business-management support, not only payment acceptance. It should help a merchant record purchases from suppliers, track stock, understand what sells quickly, see low-stock items, track costs and revenue, estimate profit, and prepare useful business/tax-related records.

Important calculations:

    Sales/revenue is not the same as profit.
    Gross profit needs a defensible cost-of-goods-sold calculation based on purchase cost/inventory costing.
    Procurement records should preserve supplier, items, quantities, unit purchase costs, total, date, and payment/reference information when available.
    Stock changes must be traceable; avoid silently mutating inventory without movement/history records.
    Tax-related records do not mean EBM approval or automatic tax compliance.

7. Notifications and accessibility

When a payment is recorded, the merchant should receive a clear notification that may include customer name (when available), items, and amount, with sound where appropriate. Example: “Elissa paid 5,000 RWF for 2 kg rice and 1 kg sugar.”

A payment notification must not automatically instruct the merchant to release goods. Payment, fulfillment, debt, advance payment, and business agreement are separate concepts.

For product selection, support a small image plus product name. Manual product creation is essential. Barcode/QR scanning is optional, not required for small merchants.
8. User roles

Architecture should eventually support role-based access for:

    Customer/buyer
    Merchant
    Hospital
    Insurer
    Investor
    Transport operator
    Platform administrator

Use least privilege. A user may have roles as designed; do not assume every role is a separate login/account.
9. Security and money safety

Security is a core requirement. Use defense in depth, not claims such as “100% unhackable.”

Required principles:

    Strong password hashing using a reputable password-hashing algorithm and appropriate parameters
    Secure session/token strategy, expiry, rotation/revocation as appropriate
    Authorization checks on every protected resource; do not rely only on frontend controls
    Input validation and safe database queries
    Rate limiting and abuse protections
    Secure secret/environment configuration; never commit secrets
    Audit logs and security events
    Least privilege and careful handling of personal/financial data
    Idempotency for payment operations
    Database transactions for money/inventory state transitions
    Immutable or append-only ledger entries for financial movements
    Reconciliation of internal records against payment-provider/bank records
    Concurrency and failure testing
    Avoid a simplistic mutable wallet balance as the sole financial record
    Do not allow client-supplied totals or status to decide a payment succeeded

For money, prefer a ledger-based design with traceable entries, balanced postings where applicable, idempotency, and explicit states. A payment provider's verified callback/status must be validated before marking external payments successful.
10. Online-only operation, low data use, and future integrations

The agreed initial IHUZA+ product is online-only. Checkout, money transfers, payment confirmation, receipts, and payment notifications require a live connection to the IHUZA+ server.

    If there is no internet connection, show a short and clear connection message and do not continue the financial operation.
    Do not queue an offline money transfer or payment for automatic execution later.
    Do not show a pending offline operation as paid, successful, or received.
    Do not implement offline payments or offline CBDC unless the developer explicitly reopens the decision and the required legal, provider, fraud-control, and reconciliation design has been approved.
    Keep data use low through compact API responses, pagination, small product thumbnails, image compression, lazy loading, caching of non-sensitive static assets, and avoiding unnecessary background requests.
    Caching used to reduce bandwidth must not be represented as offline financial capability. The server remains the source of truth for prices, totals, balances, transactions, receipts, and notifications.

Potential integrations include MTN, Airtel, banks, transport systems, EBM/tax ecosystem, and future CBDC. Treat all as integration boundaries until official access, documentation, credentials, approvals, and tests exist.
11. Revenue and fairness

Potential revenue sources to research later:

    Merchant service fees
    Business subscriptions
    Premium merchant tools
    API/integration fees
    Enterprise services
    Partner commissions
    Other lawful service fees

Do not finalize pricing without business analysis. Keep basic citizen access affordable and avoid unnecessary fees.
12. Current known technical environment

From the developer's reported environment:

    Windows 10 Pro 22H2
    HP ProBook 640 G1
    Intel Core i5-4200M @ 2.50 GHz
    RAM 8 GB
    Rust/Cargo 1.94.0
    Docker 29.6.2
    Docker Compose 5.3.1
    PostgreSQL 17 running in Docker
    Backend: Rust
    API framework: Axum 0.8.9
    Async runtime: Tokio 1.53.1
    SQLx CLI: 0.9.0
    Project root reported as C:\Users\PC\IHUZA+
    Backend reported as C:\Users\PC\IHUZA+\backend
    PostgreSQL container name: ihuza-postgres
    Container port mapping: 5432:5432
    Database: ihuza
    Local development username: ihuza
    Local development password was reported as [REDACTED] (local-only; rotate/change before production and never commit it)
    Local connection string used: postgres://ihuza:[REDACTED]@localhost:5432/ihuza

Do not repeat BIOS/virtualization setup: it was already fixed. Docker has worked, but the PostgreSQL container sometimes stops. If connection refused, first check docker ps; if stopped, start ihuza-postgres. Do not assume it is stopped without checking.
13. Reported repository structure and completed work

Reported backend files:

    Cargo.toml
    Cargo.lock
    .gitignore
    src/
    migrations/

The Rust Axum health endpoint previously worked at http://127.0.0.1:3000/health and returned IHUZA+ is healthy.
A Rust-to-PostgreSQL SQLx connection test previously succeeded and printed:
IHUZA+ connected to PostgreSQL successfully!

SQLx CLI was installed and sqlx --version returned sqlx-cli 0.9.0.

Reported migrations and database work:

    20260809130908_init_identity.sql
        pgcrypto extension
        users
        roles
        user_roles
        merchant_profiles
        role seed rows for CUSTOMER, MERCHANT, TRANSPORT_OPERATOR, ADMIN
        sqlx migrate run succeeded for this migration.
    20260810095732_create_products.sql
        products
        product_images
        merchant-scoped SKU/barcode uniqueness and one-primary-image index
        sqlx migrate run succeeded for this migration.
    20260810125637_create_invoices.sql
        invoices
        invoice_items
        SQL was provided in the conversation; user reported the migration succeeded.
        Includes merchant, optional customer, invoice number, subtotal/total, currency, payment status, invoice date, QR reference, item snapshots, quantities, unit prices, and indexes/checks.
    20260810144533_create_transactions.sql
        transactions
        SQL was provided in the conversation; user reported it worked.
        Includes optional invoice, sender/receiver, amount, currency, payment method, payment reference, status, transaction date, receipt reference, timestamps, checks and indexes.
    20260811081204_create_suppliers.sql
        Migration file was created.
        SQL was provided for suppliers: merchant ID, name, phone, email, address, tax identification number, active flag, timestamps, indexes and merchant-scoped unique tax ID.
        SQLx migration history confirmed it installed on 2026-11-02. The earlier connection refusal is historical.

Important: These are user-reported states, not verified directly against the local filesystem. If Codex has access to the repository, inspect actual files, migration names, SQLx migration history, and Git status before changing anything. Do not create duplicate migrations just because this handoff says a migration exists.
14. Current continuation point (verified 2026-11-03)

SQLx migration history confirmed that create_suppliers and the subsequent sessions, security, procurement, merchant-approval-event, admin-bootstrap-event, and unified-account-QR-reference migrations are installed. The database connection worked on 2026-11-03. The backend has customer registration/login and an authenticated merchant application endpoint. An ADMIN-only endpoint can approve a pending application, grant the MERCHANT role, and record an audit event in one database transaction. The developer ran the local bootstrap_admin command; it reported success, and a later read-only run confirmed that an ADMIN now exists. The account QR reference is a single opaque value per user. Approved merchants can create products with their own RWF prices, update an active product's price, and list their active products through authenticated, paginated endpoints. An authenticated customer can resolve an approved merchant's account QR reference to a paginated catalog; a pending merchant or regular customer account does not expose a merchant catalog. A customer checkout quote now reads current approved-merchant product prices from the server, rounds each line to two decimal places for RWF, and does not create an invoice or payment. A customer can now confirm a fresh quote to create one PENDING Smart Invoice draft with immutable item snapshots. An additive idempotency migration was installed on 2026-11-03, and retries with the same key return the original draft. Customer and merchant owners can read their invoice list and item snapshot detail through authenticated, owner-scoped endpoints; unrelated accounts cannot read them. This does not record payment or create a verified receipt. QR image generation, payment, and Receipt Vault access remain future work. A test-only RWF ledger migration was installed on 2026-11-03 with balanced append-only postings and an idempotent test transfer function. Its database integration test passed and rolled back all fixture data. This ledger is isolated from invoices and transactions, has no API, and does not enable real payments or receipts. backend/docs/PAYMENTS_DESIGN.md records the remaining provider, verification, reconciliation, and production-ledger requirements. Verify the live repository and migration history again before taking further action.

An authenticated QR destination resolver was added on 2026-11-03: `GET /qr/{reference}` identifies an approved merchant checkout or a person transfer destination using the same account QR. It shows a display name and RWF, rejects self-scans, and does not expose wallet balance or debit funds. Pending merchants do not appear as approved merchants. The relevant database integration test passed. A person-to-person transfer API and a production wallet-ledger service are still pending.

The additive `20261103180000_create_wallet_ledger_foundation.sql` migration was installed on 2026-11-03. It creates RWF wallet accounts linked to users, immutable ledger entries, checked balanced batches, and a derived balance view. Its rollback-only database test rejected unbalanced and unbacked postings. This is a dormant production-schema foundation: no account provisioning, funding, transfer API, verified receipt, or reconciliation exists. The earlier test ledger remains separate. Do not claim the wallet can hold real customer money yet.

The additive `20261103181500_guard_wallet_batch_creation.sql` migration was also installed after a review found that a direct `POSTED` batch insert could otherwise bypass the posting transition check. It now requires all new batches to begin as `DRAFT`; the database test covers this. The runtime application database role still needs to be separated from the schema owner before any real funds are handled.

The backend now has authenticated `POST /wallet` to provision one zero-value RWF wallet for the signed-in CUSTOMER, and `GET /wallet` to read only that customer's derived balance and status. Both check the server-side session and CUSTOMER role. Repeated provisioning returns the same account and does not unfreeze a FROZEN wallet. The response says `money_operations_enabled: false`. The rollback-only database test passed. This still does not enable funding, transfers, payments, or holding real customer money.

On 2026-11-03 the local database `ihuza` role was confirmed to be a superuser and owner of the wallet tables. The additive `20261103200000_create_limited_runtime_role.sql` migration created a separate `ihuza_app` role with narrowly granted current API permissions, no wallet-ledger write permission, and no schema ownership. A random local credential was saved only in ignored `backend/.env` as `APP_DATABASE_URL`; `DATABASE_URL` remains for migrations and bootstrap_admin. The backend now requires the limited connection and refuses to start with the superuser. `backend/docs/DATABASE_ROLES.md` records setup and limitations. The runtime-role rollback test passed; no real-money operation is enabled.

On 2026-11-04 PostgreSQL connection and SQLx migration history worked again. `create_suppliers` and all prior migrations remained installed. The additive `20261104100000_create_transfer_intents.sql` migration installed a person-to-person transfer-intent table limited to `DRAFT`. Authenticated `POST /wallet/transfer-intents` accepts the recipient's single account QR reference, a positive RWF amount, and an idempotency key; it requires the sender's active zero-value wallet, rejects self-scans and approved merchant destinations, and reports `money_moved: false`. The API role may insert and read drafts but cannot update them or write wallet ledger entries. This is not a money transfer, payment confirmation, reservation, or receipt. Merchant invoice payment intents, real funding, wallet posting, reconciliation, and customer-facing receipt history remain pending. The rollback-only runtime-role transfer-draft test and bearer-token route test passed.

Later on 2026-11-04 the additive `20261104110000_link_transfer_intents_to_invoices.sql` migration installed. Authenticated `POST /wallet/invoice-intents` creates one `MERCHANT_INVOICE` draft for a customer-owned PENDING Smart Invoice from the approved merchant checkout. The server reads the invoice total, merchant, and RWF currency; the customer cannot supply a payment amount. Reusing the same idempotency key returns the same draft; another draft for the same invoice is rejected. The rollback-only database test confirmed ownership, server total, idempotency, unchanged invoice status, and unchanged wallet balance. This still does not execute payment, create a verified receipt, or enable real funds.

Authenticated `GET /wallet/intents` now lists only the signed-in CUSTOMER's own `DRAFT` intents with bounded pagination. It distinguishes person and merchant-invoice intents, shows the recipient and optional invoice reference, and never treats a draft as a completed payment. The rollback-only database test covers both kinds, pagination, and another user's empty result. No schema change was needed for this read endpoint.

Authenticated `GET /account/qr.svg` renders the existing single opaque account QR reference as an SVG image; it does not introduce another QR or include price, invoice, balance, provider, or secret data. It requires the owner's live session. The image by itself does not implement a scanner, money transfer, or payment. `qrcodegen` was added to Cargo dependencies for QR encoding.

15. Suggested development roadmap

Do not blindly create all tables at once. Inspect current schema first and design relationships before adding new migrations.

    Foundation: Rust/Axum/Tokio/PostgreSQL/SQLx, configuration, logging, health endpoint.
    Identity and access: users, roles, registration/login, password hashing, sessions/tokens, authorization, rate limits, audit/security events.
    Merchant Assistant: merchant profiles, suppliers, procurement and procurement items, stock/inventory movements, purchase cost, sales, revenue, profit, reports, tax-related information.
    Smart Invoice: invoice and invoice-item semantics, reliable totals, invoice states, QR reference.
    Payments: transaction states, idempotency, payment orchestration/provider abstraction, ledger and reconciliation. Avoid balance-only design.
    Receipt Vault: customer/merchant receipt access, searchable history, durable historical snapshots, tax/EBM integration boundary.
    Notifications and automation: payment notifications, item summary, sound, failure/security alerts; do not equate payment with fulfillment.
    Transport payments: operator integration, passenger payment, QR/cards as appropriate, receipts and queue reduction.
    External payment integrations: banks, MTN/Airtel, approved rails, CBDC only when officially available and approved.
    Frontend: accessible, modern, mobile-friendly UI.
    Testing: unit, integration, API, database, security, concurrency, payment failures, inventory and recovery.
    Deployment: only after local system and security checks are stable.

The order can be adjusted based on the existing implementation, but explain architectural reasons before changing the agreed plan.
16. Product/stock/accounting rules

    Merchants own their product catalog and choose their own prices.
    Same product may have different prices at different merchants.
    Historical invoice item name/price must not change when the product catalog changes.
    Inventory should be tracked with auditable movements, not only a manually edited number.
    Define the costing method before implementing profit calculations; do not report precise profit if purchase costs/stock movements are incomplete.
    Distinguish revenue, gross profit, net profit, expenses, payment receipts, supplier invoices, and tax records.
    A purchase/procurement record should not automatically imply it has been paid in full.
    Partial payments, advances, debts, refunds, cancelled transactions, and fulfillment states must be represented accurately.

17. Development process for Codex

When beginning work in the repository:

    Read this file.
    Inspect git status, Cargo.toml, src/, migrations/, and current migration history.
    Identify the exact current state; don't trust the handoff over the repository if they differ—report the difference.
    Do not commit secrets or the local database password.
    Do not delete/rewrite applied migrations casually. If schema changes are needed after application, add a new corrective migration.
    Before any destructive database command, explain the risk and obtain explicit confirmation.
    Prefer one small change at a time and run formatting/tests/checks where appropriate.
    Report files changed, commands run, test results, and any unverified assumptions.
    Keep developer-facing explanations in Kinyarwanda and concise.
    If a requirement is unclear or conflicts with the agreed architecture, pause and ask rather than silently deciding.

18. First prompt to Codex

After placing this file in the backend repository root (preferably named AGENTS.md so Codex can use it as repository instructions), ask Codex:

“Read AGENTS.md completely. Then inspect the actual repository, Git status, migrations, and database connection/migration state. Do not rewrite or scaffold the project. Compare the actual state with the handoff and report: (1) what is confirmed complete, (2) what is incomplete or failed, (3) the safest exact next step. Start with the pending create_suppliers migration only if the repository and database confirm it is pending. Explain briefly in Kinyarwanda and do not make broad changes without approval.”
⋮⋮
⋮⋮
