use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use sqlx::postgres::PgPoolOptions;

#[tokio::test]
#[ignore = "requires APP_DATABASE_URL; fixtures are rolled back"]
async fn transfer_draft_cannot_move_money_or_be_changed_by_runtime_role() {
    dotenvy::dotenv().ok();
    let database_url = env::var("APP_DATABASE_URL").expect("APP_DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .expect("runtime role should connect");
    let can_update: bool = sqlx::query_scalar(
        "SELECT has_table_privilege(current_user, 'wallet_transfer_intents', 'UPDATE')",
    )
    .fetch_one(&pool)
    .await
    .expect("permissions should be readable");
    assert!(!can_update, "runtime cannot mark a draft as paid");

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    let mut tx = pool.begin().await.expect("test transaction should begin");
    let sender: String = sqlx::query_scalar(
        "INSERT INTO users (phone_number, password_hash, first_name)
         VALUES ($1, 'draft-test-only-hash', 'Sender') RETURNING id::text",
    )
    .bind(format!("draft-sender-{suffix}"))
    .fetch_one(&mut *tx)
    .await
    .expect("sender should be created");
    let recipient: String = sqlx::query_scalar(
        "INSERT INTO users (phone_number, password_hash, first_name)
         VALUES ($1, 'draft-test-only-hash', 'Recipient') RETURNING id::text",
    )
    .bind(format!("draft-recipient-{suffix}"))
    .fetch_one(&mut *tx)
    .await
    .expect("recipient should be created");
    sqlx::query("INSERT INTO wallet_accounts (user_id) VALUES ($1::uuid)")
        .bind(&sender)
        .execute(&mut *tx)
        .await
        .expect("zero wallet should open");
    let key = format!("draft-{suffix}");
    let hash = "a".repeat(64);
    let inserted: String = sqlx::query_scalar(
        "INSERT INTO wallet_transfer_intents
         (sender_user_id, recipient_user_id, amount, request_key, request_hash)
         VALUES ($1::uuid, $2::uuid, 1.00, $3, $4) RETURNING status",
    )
    .bind(&sender)
    .bind(&recipient)
    .bind(&key)
    .bind(&hash)
    .fetch_one(&mut *tx)
    .await
    .expect("draft should be created");
    assert_eq!(inserted, "DRAFT");
    let repeated = sqlx::query(
        "INSERT INTO wallet_transfer_intents
         (sender_user_id, recipient_user_id, amount, request_key, request_hash)
         VALUES ($1::uuid, $2::uuid, 1.00, $3, $4)
         ON CONFLICT (sender_user_id, request_key) DO NOTHING",
    )
    .bind(&sender)
    .bind(&recipient)
    .bind(&key)
    .bind(&hash)
    .execute(&mut *tx)
    .await
    .expect("retry should not create another draft");
    assert_eq!(repeated.rows_affected(), 0);
    let (balance, intent_count): (String, i64) = sqlx::query_as(
        "SELECT b.balance::text, (SELECT COUNT(*) FROM wallet_transfer_intents i
           WHERE i.sender_user_id = $1::uuid)
         FROM wallet_balances b WHERE b.user_id = $1::uuid",
    )
    .bind(&sender)
    .fetch_one(&mut *tx)
    .await
    .expect("wallet state should be readable");
    assert_eq!(balance, "0.00");
    assert_eq!(intent_count, 1);
    tx.rollback().await.expect("fixtures should roll back");
}
