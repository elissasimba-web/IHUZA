use std::{env, error::Error, io};

use sqlx::{PgPool, Postgres, Transaction, postgres::PgPoolOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").map_err(|_| {
        io::Error::other("DATABASE_URL is missing; set it in PowerShell or backend/.env")
    })?;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;

    let database_name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await?;
    println!("Database: {database_name}");
    let admin_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
             SELECT 1 FROM user_roles ur
             JOIN roles r ON r.id = ur.role_id
             WHERE r.name = 'ADMIN'
         )",
    )
    .fetch_one(&pool)
    .await?;
    if admin_exists {
        return Err(io::Error::other("An ADMIN already exists; bootstrap is closed").into());
    }
    println!("Enter the phone number of an existing, active IHUZA+ account:");
    let mut phone_number = String::new();
    io::stdin().read_line(&mut phone_number)?;
    let phone_number = phone_number.trim();
    if phone_number.is_empty() {
        return Err(io::Error::other("Phone number is required").into());
    }

    let account = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT u.first_name, u.last_name
         FROM users u
         WHERE u.phone_number = $1 AND u.is_active = TRUE
           AND EXISTS (
               SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id
               WHERE ur.user_id = u.id AND r.name = 'CUSTOMER'
           )",
    )
    .bind(phone_number)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| io::Error::other("No active account has that phone number"))?;
    println!(
        "Account: {} {} ({phone_number})",
        account.0,
        account.1.unwrap_or_default()
    );
    println!("Type APPROVE to grant this account the initial ADMIN role:");
    let mut confirmation = String::new();
    io::stdin().read_line(&mut confirmation)?;
    if confirmation.trim() != "APPROVE" {
        return Err(io::Error::other("No role was granted").into());
    }

    bootstrap_admin(&pool, phone_number).await?;
    println!("Initial ADMIN role granted and audited.");
    Ok(())
}

async fn bootstrap_admin(pool: &PgPool, phone_number: &str) -> Result<(), Box<dyn Error>> {
    let mut transaction = pool.begin().await?;
    grant_initial_admin(&mut transaction, phone_number).await?;
    transaction.commit().await?;
    Ok(())
}

async fn grant_initial_admin(
    transaction: &mut Transaction<'_, Postgres>,
    phone_number: &str,
) -> Result<(), Box<dyn Error>> {
    sqlx::query("SELECT pg_advisory_xact_lock(2026110216)")
        .execute(&mut **transaction)
        .await?;

    let admin_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
             SELECT 1 FROM user_roles ur
             JOIN roles r ON r.id = ur.role_id
             WHERE r.name = 'ADMIN'
         )",
    )
    .fetch_one(&mut **transaction)
    .await?;
    if admin_exists {
        return Err(io::Error::other("An ADMIN already exists; bootstrap is closed").into());
    }

    let user_id: String = sqlx::query_scalar(
        "SELECT u.id::text FROM users u
         WHERE u.phone_number = $1 AND u.is_active = TRUE
           AND EXISTS (
               SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id
               WHERE ur.user_id = u.id AND r.name = 'CUSTOMER'
           )
         FOR UPDATE",
    )
    .bind(phone_number)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or_else(|| io::Error::other("The selected active account no longer exists"))?;

    let assigned = sqlx::query(
        "INSERT INTO user_roles (user_id, role_id)
         SELECT $1::uuid, id FROM roles WHERE name = 'ADMIN'
         ON CONFLICT DO NOTHING",
    )
    .bind(&user_id)
    .execute(&mut **transaction)
    .await?;
    if assigned.rows_affected() != 1 {
        return Err(io::Error::other("ADMIN role could not be assigned").into());
    }

    sqlx::query(
        "INSERT INTO security_events (user_id, event_type, metadata)
         VALUES ($1::uuid, 'ADMIN_BOOTSTRAPPED', $2)",
    )
    .bind(&user_id)
    .bind(serde_json::json!({
        "method": "local_cli",
        "admin_user_id": user_id,
    }))
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    #[ignore = "requires DATABASE_URL and rolls back isolated test data"]
    async fn admin_bootstrap_grants_once_or_rejects_existing_admin() {
        dotenvy::dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("test database connection should succeed");
        let mut transaction = pool.begin().await.expect("transaction should begin");
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let phone_number = format!("bootstrap-test-{suffix}");
        let user_id: String = sqlx::query_scalar(
            "INSERT INTO users (phone_number, password_hash, first_name)
             VALUES ($1, 'test-only-hash', 'Bootstrap Test') RETURNING id::text",
        )
        .bind(&phone_number)
        .fetch_one(&mut *transaction)
        .await
        .expect("test user should be created");
        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id)
             SELECT $1::uuid, id FROM roles WHERE name = 'CUSTOMER'",
        )
        .bind(&user_id)
        .execute(&mut *transaction)
        .await
        .expect("customer role should be assigned");

        let admin_already_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                 SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id
                 WHERE r.name = 'ADMIN'
             )",
        )
        .fetch_one(&mut *transaction)
        .await
        .expect("admin state should be queryable");
        if admin_already_exists {
            assert!(
                grant_initial_admin(&mut transaction, &phone_number)
                    .await
                    .is_err()
            );
            let candidate_role_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM user_roles ur JOIN roles r ON r.id = ur.role_id
                 WHERE ur.user_id = $1::uuid AND r.name = 'ADMIN'",
            )
            .bind(&user_id)
            .fetch_one(&mut *transaction)
            .await
            .expect("candidate role should be queryable");
            let candidate_audit_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM security_events
                 WHERE user_id = $1::uuid AND event_type = 'ADMIN_BOOTSTRAPPED'",
            )
            .bind(&user_id)
            .fetch_one(&mut *transaction)
            .await
            .expect("candidate audit should be queryable");
            assert_eq!((candidate_role_count, candidate_audit_count), (0, 0));
            transaction
                .rollback()
                .await
                .expect("test changes should roll back");
            return;
        }

        grant_initial_admin(&mut transaction, &phone_number)
            .await
            .expect("first admin should be granted");
        let role_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_roles ur JOIN roles r ON r.id = ur.role_id
             WHERE ur.user_id = $1::uuid AND r.name = 'ADMIN'",
        )
        .bind(&user_id)
        .fetch_one(&mut *transaction)
        .await
        .expect("admin role should be queryable");
        let audit_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM security_events
             WHERE user_id = $1::uuid AND event_type = 'ADMIN_BOOTSTRAPPED'
               AND metadata->>'admin_user_id' = $1",
        )
        .bind(&user_id)
        .fetch_one(&mut *transaction)
        .await
        .expect("audit event should be queryable");
        assert_eq!((role_count, audit_count), (1, 1));
        assert!(
            grant_initial_admin(&mut transaction, &phone_number)
                .await
                .is_err()
        );

        transaction
            .rollback()
            .await
            .expect("test changes should roll back");
    }
}
