use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use sqlx::postgres::PgPoolOptions;

#[tokio::test]
#[ignore = "requires DATABASE_URL; wallet fixture rows are rolled back"]
async fn wallet_ledger_rejects_unbacked_or_unbalanced_value() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .expect("test database connection should succeed");
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    let mut tx = pool.begin().await.expect("test transaction should begin");

    let mut accounts = Vec::new();
    for name in ["Alice", "Bob"] {
        let user_id: String = sqlx::query_scalar(
            "INSERT INTO users (phone_number, password_hash, first_name)
             VALUES ($1, 'wallet-test-only-hash', $2) RETURNING id::text",
        )
        .bind(format!("wallet-{name}-{suffix}"))
        .bind(name)
        .fetch_one(&mut *tx)
        .await
        .expect("test user should be created");
        let account_id: String = sqlx::query_scalar(
            "INSERT INTO wallet_accounts (user_id)
             VALUES ($1::uuid) RETURNING id::text",
        )
        .bind(&user_id)
        .fetch_one(&mut *tx)
        .await
        .expect("test wallet account should be created");
        accounts.push(account_id);
    }

    sqlx::query("SAVEPOINT forged_posted")
        .execute(&mut *tx)
        .await
        .expect("savepoint should be created");
    let forged_posted = sqlx::query(
        "INSERT INTO wallet_ledger_batches (
             sender_account_id, recipient_account_id, amount,
             request_key, request_hash, status, posted_at
         ) VALUES ($1::uuid, $2::uuid, 20.00, $3, repeat('a', 64), 'POSTED', NOW())",
    )
    .bind(&accounts[0])
    .bind(&accounts[1])
    .bind(format!("walletforged{suffix}"))
    .execute(&mut *tx)
    .await;
    assert!(forged_posted.is_err(), "a batch cannot start as POSTED");
    sqlx::query("ROLLBACK TO SAVEPOINT forged_posted")
        .execute(&mut *tx)
        .await
        .expect("forged posting should roll back");

    let batch_id: String = sqlx::query_scalar(
        "INSERT INTO wallet_ledger_batches (
             sender_account_id, recipient_account_id, amount, request_key, request_hash
         ) VALUES ($1::uuid, $2::uuid, 20.00, $3, repeat('a', 64))
         RETURNING id::text",
    )
    .bind(&accounts[0])
    .bind(&accounts[1])
    .bind(format!("walletunfunded{suffix}"))
    .fetch_one(&mut *tx)
    .await
    .expect("draft batch should be created");

    sqlx::query(
        "INSERT INTO wallet_ledger_entries (batch_id, account_id, amount)
         VALUES ($1::uuid, $2::uuid, -20.00)",
    )
    .bind(&batch_id)
    .bind(&accounts[0])
    .execute(&mut *tx)
    .await
    .expect("draft sender entry should be created");

    sqlx::query("SAVEPOINT unbalanced")
        .execute(&mut *tx)
        .await
        .expect("savepoint should be created");
    let unbalanced =
        sqlx::query("UPDATE wallet_ledger_batches SET status = 'POSTED' WHERE id = $1::uuid")
            .bind(&batch_id)
            .execute(&mut *tx)
            .await;
    assert!(unbalanced.is_err(), "one-sided wallet entry cannot post");
    sqlx::query("ROLLBACK TO SAVEPOINT unbalanced")
        .execute(&mut *tx)
        .await
        .expect("failed posting should roll back");

    sqlx::query(
        "INSERT INTO wallet_ledger_entries (batch_id, account_id, amount)
         VALUES ($1::uuid, $2::uuid, 20.00)",
    )
    .bind(&batch_id)
    .bind(&accounts[1])
    .execute(&mut *tx)
    .await
    .expect("draft recipient entry should be created");
    sqlx::query("SAVEPOINT unfunded")
        .execute(&mut *tx)
        .await
        .expect("savepoint should be created");
    let unfunded =
        sqlx::query("UPDATE wallet_ledger_batches SET status = 'POSTED' WHERE id = $1::uuid")
            .bind(&batch_id)
            .execute(&mut *tx)
            .await;
    assert!(unfunded.is_err(), "a wallet cannot spend unbacked value");
    sqlx::query("ROLLBACK TO SAVEPOINT unfunded")
        .execute(&mut *tx)
        .await
        .expect("failed posting should roll back");

    sqlx::query("SAVEPOINT immutable")
        .execute(&mut *tx)
        .await
        .expect("savepoint should be created");
    let mutation =
        sqlx::query("UPDATE wallet_ledger_entries SET amount = 30.00 WHERE batch_id = $1::uuid")
            .bind(&batch_id)
            .execute(&mut *tx)
            .await;
    assert!(mutation.is_err(), "wallet entries must be append-only");
    sqlx::query("ROLLBACK TO SAVEPOINT immutable")
        .execute(&mut *tx)
        .await
        .expect("failed mutation should roll back");

    let balance: String =
        sqlx::query_scalar("SELECT balance::text FROM wallet_balances WHERE account_id = $1::uuid")
            .bind(&accounts[0])
            .fetch_one(&mut *tx)
            .await
            .expect("wallet balance should be derived from posted entries");
    assert_eq!(balance, "0.00");

    tx.rollback().await.expect("test fixtures should roll back");
    let remaining: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM wallet_ledger_batches WHERE request_key = $1")
            .bind(format!("walletunfunded{suffix}"))
            .fetch_one(&pool)
            .await
            .expect("test rollback should be verifiable");
    assert_eq!(remaining, 0);
}
