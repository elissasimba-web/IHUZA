use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use sqlx::postgres::PgPoolOptions;

#[tokio::test]
#[ignore = "requires APP_DATABASE_URL; all fixture data is rolled back"]
async fn runtime_role_can_provision_but_cannot_modify_wallet_ledger() {
    dotenvy::dotenv().ok();
    let database_url = env::var("APP_DATABASE_URL").expect("APP_DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .expect("runtime role should connect");
    let (is_limited, can_provision, can_read_balance, can_post, can_change_accounts): (
        bool,
        bool,
        bool,
        bool,
        bool,
    ) = sqlx::query_as(
        "SELECT NOT r.rolsuper AND NOT r.rolcreatedb AND NOT r.rolcreaterole
                AND NOT has_schema_privilege(current_user, 'public', 'CREATE'),
                has_table_privilege(current_user, 'wallet_accounts', 'INSERT'),
                has_table_privilege(current_user, 'wallet_balances', 'SELECT'),
                has_table_privilege(current_user, 'wallet_ledger_entries', 'INSERT'),
                has_table_privilege(current_user, 'wallet_accounts', 'UPDATE')
         FROM pg_roles r WHERE r.rolname = current_user",
    )
    .fetch_one(&pool)
    .await
    .expect("runtime permissions should be readable");
    assert!(is_limited && can_provision && can_read_balance);
    assert!(!can_post && !can_change_accounts);

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    let mut tx = pool.begin().await.expect("test transaction should begin");
    let user_id: String = sqlx::query_scalar(
        "INSERT INTO users (phone_number, password_hash, first_name)
         VALUES ($1, 'runtime-role-test-only-hash', 'Runtime Test') RETURNING id::text",
    )
    .bind(format!("runtime-role-{suffix}"))
    .fetch_one(&mut *tx)
    .await
    .expect("runtime role should create users");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id)
         SELECT $1::uuid, id FROM roles WHERE name = 'CUSTOMER'",
    )
    .bind(&user_id)
    .execute(&mut *tx)
    .await
    .expect("runtime role should grant customer role");
    let account_id: String = sqlx::query_scalar(
        "INSERT INTO wallet_accounts (user_id) VALUES ($1::uuid) RETURNING id::text",
    )
    .bind(&user_id)
    .fetch_one(&mut *tx)
    .await
    .expect("runtime role should create an empty wallet");
    let balance: String =
        sqlx::query_scalar("SELECT balance::text FROM wallet_balances WHERE account_id = $1::uuid")
            .bind(&account_id)
            .fetch_one(&mut *tx)
            .await
            .expect("runtime role should read its test wallet balance");
    assert_eq!(balance, "0.00");

    tx.rollback().await.expect("test data should roll back");
}
