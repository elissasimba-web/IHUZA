use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use sqlx::postgres::PgPoolOptions;

#[tokio::test]
#[ignore = "requires DATABASE_URL; all test ledger rows are rolled back"]
async fn test_ledger_posts_balanced_idempotent_transfers_and_rejects_bad_changes() {
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
    let source_ref = format!("testsource{suffix}");
    let alice_ref = format!("testalice{suffix}");
    let bob_ref = format!("testbob{suffix}");
    let funding_key = format!("testfunding{suffix}");
    let transfer_key = format!("testtransfer{suffix}");
    let mut tx = pool.begin().await.expect("test transaction should begin");

    let mut account_ids = Vec::new();
    for (account_ref, kind) in [
        (&source_ref, "TEST_SOURCE"),
        (&alice_ref, "USER"),
        (&bob_ref, "USER"),
    ] {
        let id: String = sqlx::query_scalar(
            "INSERT INTO test_ledger_accounts (account_ref, kind)
             VALUES ($1, $2) RETURNING id::text",
        )
        .bind(account_ref)
        .bind(kind)
        .fetch_one(&mut *tx)
        .await
        .expect("test account should be created");
        account_ids.push(id);
    }

    let funding_id: String =
        sqlx::query_scalar("SELECT test_ledger_transfer($1, $2, $3::numeric, $4)::text")
            .bind(&source_ref)
            .bind(&alice_ref)
            .bind("100.00")
            .bind(&funding_key)
            .fetch_one(&mut *tx)
            .await
            .expect("test funding should post");
    let funding_retry_id: String =
        sqlx::query_scalar("SELECT test_ledger_transfer($1, $2, $3::numeric, $4)::text")
            .bind(&source_ref)
            .bind(&alice_ref)
            .bind("100.00")
            .bind(&funding_key)
            .fetch_one(&mut *tx)
            .await
            .expect("same key should return the existing batch");
    assert_eq!(funding_id, funding_retry_id);

    let transfer_id: String =
        sqlx::query_scalar("SELECT test_ledger_transfer($1, $2, $3::numeric, $4)::text")
            .bind(&alice_ref)
            .bind(&bob_ref)
            .bind("40.00")
            .bind(&transfer_key)
            .fetch_one(&mut *tx)
            .await
            .expect("funded user should transfer test value");
    let (entry_count, net): (i64, String) = sqlx::query_as(
        "SELECT COUNT(*), SUM(amount)::text FROM test_ledger_entries
         WHERE batch_id = $1::uuid",
    )
    .bind(&transfer_id)
    .fetch_one(&mut *tx)
    .await
    .expect("posted batch should be readable");
    assert_eq!(entry_count, 2);
    assert_eq!(net, "0.00");

    for (account_id, expected) in [
        (&account_ids[0], "-100.00"),
        (&account_ids[1], "60.00"),
        (&account_ids[2], "40.00"),
    ] {
        let balance: String = sqlx::query_scalar(
            "SELECT balance::text FROM test_ledger_balances WHERE account_id = $1::uuid",
        )
        .bind(account_id)
        .fetch_one(&mut *tx)
        .await
        .expect("posted balance should be readable");
        assert_eq!(balance, expected);
    }

    sqlx::query("SAVEPOINT reused_key")
        .execute(&mut *tx)
        .await
        .expect("savepoint should be created");
    let reused_key = sqlx::query_scalar::<_, String>(
        "SELECT test_ledger_transfer($1, $2, $3::numeric, $4)::text",
    )
    .bind(&alice_ref)
    .bind(&bob_ref)
    .bind("41.00")
    .bind(&transfer_key)
    .fetch_one(&mut *tx)
    .await;
    assert!(reused_key.is_err(), "a reused key cannot change the amount");
    sqlx::query("ROLLBACK TO SAVEPOINT reused_key")
        .execute(&mut *tx)
        .await
        .expect("failed statement should roll back to savepoint");

    sqlx::query("SAVEPOINT overspend")
        .execute(&mut *tx)
        .await
        .expect("savepoint should be created");
    let overspend = sqlx::query_scalar::<_, String>(
        "SELECT test_ledger_transfer($1, $2, $3::numeric, $4)::text",
    )
    .bind(&alice_ref)
    .bind(&bob_ref)
    .bind("61.00")
    .bind(format!("testoverspend{suffix}"))
    .fetch_one(&mut *tx)
    .await;
    assert!(
        overspend.is_err(),
        "test user cannot spend beyond posted balance"
    );
    sqlx::query("ROLLBACK TO SAVEPOINT overspend")
        .execute(&mut *tx)
        .await
        .expect("failed statement should roll back to savepoint");

    sqlx::query("SAVEPOINT unbalanced")
        .execute(&mut *tx)
        .await
        .expect("savepoint should be created");
    let unbalanced_id: String = sqlx::query_scalar(
        "INSERT INTO test_ledger_batches (request_key, request_hash)
         VALUES ($1, repeat('a', 64)) RETURNING id::text",
    )
    .bind(format!("testunbalanced{suffix}"))
    .fetch_one(&mut *tx)
    .await
    .expect("draft batch should be created");
    sqlx::query(
        "INSERT INTO test_ledger_entries (batch_id, account_id, amount)
         VALUES ($1::uuid, $2::uuid, 1.00)",
    )
    .bind(&unbalanced_id)
    .bind(&account_ids[1])
    .execute(&mut *tx)
    .await
    .expect("draft entry should be created");
    let unbalanced =
        sqlx::query("UPDATE test_ledger_batches SET status = 'POSTED' WHERE id = $1::uuid")
            .bind(&unbalanced_id)
            .execute(&mut *tx)
            .await;
    assert!(unbalanced.is_err(), "unbalanced batch cannot post");
    sqlx::query("ROLLBACK TO SAVEPOINT unbalanced")
        .execute(&mut *tx)
        .await
        .expect("failed batch should roll back to savepoint");

    sqlx::query("SAVEPOINT immutable")
        .execute(&mut *tx)
        .await
        .expect("savepoint should be created");
    let mutation =
        sqlx::query("UPDATE test_ledger_entries SET amount = 1.00 WHERE batch_id = $1::uuid")
            .bind(&transfer_id)
            .execute(&mut *tx)
            .await;
    assert!(mutation.is_err(), "posted entries must be append-only");
    sqlx::query("ROLLBACK TO SAVEPOINT immutable")
        .execute(&mut *tx)
        .await
        .expect("failed mutation should roll back to savepoint");

    tx.rollback()
        .await
        .expect("isolated ledger rows should roll back");
    let remaining: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM test_ledger_batches WHERE request_key = $1")
            .bind(&funding_key)
            .fetch_one(&pool)
            .await
            .expect("test cleanup should be verifiable");
    assert_eq!(remaining, 0);
}
