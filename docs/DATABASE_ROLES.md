# Database roles for local IHUZA+ development

The SQLx migration connection (`DATABASE_URL`) owns the schema. The HTTP backend must use a different connection (`APP_DATABASE_URL`) with the limited `ihuza_app` role. Both values belong only in the ignored local `.env` or a secret manager; never commit credentials.

Migration `20261103200000_create_limited_runtime_role.sql` creates `ihuza_app` without login and grants only the current API's table privileges. Local setup then gives it a randomly generated password and enables login outside the migration. Run `sqlx migrate run` with the owner connection before starting the backend. The local developer machine was configured and tested on 2026-11-03; another environment needs its own secret setup.

The backend refuses to start when `APP_DATABASE_URL` points to a superuser, a role with schema creation rights, or a role that can write wallet ledger batches/entries or change wallet account status. The runtime role can provision an empty wallet and read its derived balance. It cannot post transfers, edit ledger history, or change schema. `bootstrap_admin` and SQLx migrations still use the owner connection, never the runtime one.

After each future migration, review and explicitly grant only the new privileges the API needs. Keep a separate migration connection. Before handling real funds, further reduce broad identity-table write permissions and use narrowly scoped, audited posting operations; test role privileges, idempotency, concurrency, and reconciliation. This local role separation is one control, not approval to operate a real-money wallet.

The role is cluster-wide. The current role-creation migration assumes `ihuza_app` does not already exist in the target PostgreSQL cluster; a second database in the same cluster needs a reviewed provisioning plan before migrations run there. Do not edit the already-applied migration or SQLx history to work around that.
