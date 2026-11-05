use axum::{
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    routing::post,
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ihuza_wallet_proof::{verify_challenge, Challenge, TESTNET_PASSPHRASE};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
}

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<ErrorBody>)>;

fn error(status: StatusCode, message: &'static str) -> (StatusCode, Json<ErrorBody>) {
    (status, Json(ErrorBody { error: message }))
}

fn internal_error(_: sqlx::Error) -> (StatusCode, Json<ErrorBody>) {
    error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Habaye ikibazo cya server.",
    )
}

fn unix_now() -> Result<u64, (StatusCode, Json<ErrorBody>)> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|time| time.as_secs())
        .map_err(|_| {
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Igihe cya server nticyemewe.",
            )
        })
}

#[derive(Deserialize)]
struct ChallengeRequest {
    address: String,
}

#[derive(Serialize)]
struct ChallengeResponse {
    challenge_id: String,
    address: String,
    network_passphrase: &'static str,
    expires_at: u64,
    message: String,
}

async fn issue_challenge(
    State(state): State<AppState>,
    Json(input): Json<ChallengeRequest>,
) -> ApiResult<ChallengeResponse> {
    let challenge = Challenge::issue(&input.address, unix_now()?)
        .map_err(|_| error(StatusCode::BAD_REQUEST, "Aderesi ya wallet ntiyemewe."))?;
    let mut tx = state.pool.begin().await.map_err(internal_error)?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(&challenge.address)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    let pending: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM wallet_challenges WHERE stellar_address = $1 AND consumed_at IS NULL AND expires_at > now()",
    )
    .bind(&challenge.address)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal_error)?;
    if pending >= 3 {
        return Err(error(
            StatusCode::TOO_MANY_REQUESTS,
            "Tegereza challenge iriho irangire.",
        ));
    }
    let challenge_id: String = sqlx::query_scalar(
        "INSERT INTO wallet_challenges (network_passphrase, stellar_address, nonce_hex, issued_at, expires_at, purpose) VALUES ($1, $2, $3, to_timestamp($4::double precision), to_timestamp($5::double precision), 'authenticate') RETURNING id::text",
    )
    .bind(TESTNET_PASSPHRASE)
    .bind(&challenge.address)
    .bind(&challenge.nonce_hex)
    .bind(challenge.issued_at as i64)
    .bind(challenge.expires_at as i64)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal_error)?;
    sqlx::query("INSERT INTO security_events (challenge_id, event_kind) VALUES ($1::uuid, 'challenge_issued')")
        .bind(&challenge_id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    tx.commit().await.map_err(internal_error)?;
    Ok(Json(ChallengeResponse {
        challenge_id,
        address: challenge.address.clone(),
        network_passphrase: TESTNET_PASSPHRASE,
        expires_at: challenge.expires_at,
        message: challenge.message(),
    }))
}

#[derive(Deserialize)]
struct ProofRequest {
    challenge_id: String,
    signature_base64: String,
}

#[derive(Serialize)]
struct ProofResponse {
    account_id: String,
    qr_reference: String,
    status: &'static str,
}

async fn verify_proof(
    State(state): State<AppState>,
    Json(input): Json<ProofRequest>,
) -> ApiResult<ProofResponse> {
    if input.challenge_id.len() != 36 || input.signature_base64.len() > 200 {
        return Err(error(StatusCode::BAD_REQUEST, "Proof ntiyemewe."));
    }
    let mut tx = state.pool.begin().await.map_err(internal_error)?;
    let row = sqlx::query(
        "SELECT stellar_address, nonce_hex, extract(epoch from issued_at)::bigint AS issued_at, extract(epoch from expires_at)::bigint AS expires_at, consumed_at IS NOT NULL AS consumed FROM wallet_challenges WHERE id::text = $1 FOR UPDATE",
    )
    .bind(&input.challenge_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| error(StatusCode::BAD_REQUEST, "Challenge ntiyemewe."))?;
    let consumed: bool = row.try_get("consumed").map_err(internal_error)?;
    if consumed {
        return Err(error(StatusCode::CONFLICT, "Challenge yamaze gukoreshwa."));
    }
    let issued_at: i64 = row.try_get("issued_at").map_err(internal_error)?;
    let expires_at: i64 = row.try_get("expires_at").map_err(internal_error)?;
    let challenge = Challenge {
        address: row.try_get("stellar_address").map_err(internal_error)?,
        nonce_hex: row.try_get("nonce_hex").map_err(internal_error)?,
        issued_at: issued_at
            .try_into()
            .map_err(|_| error(StatusCode::BAD_REQUEST, "Challenge ntiyemewe."))?,
        expires_at: expires_at
            .try_into()
            .map_err(|_| error(StatusCode::BAD_REQUEST, "Challenge ntiyemewe."))?,
    };
    verify_challenge(&challenge, unix_now()?, &input.signature_base64).map_err(|_| {
        error(
            StatusCode::UNAUTHORIZED,
            "Umukono cyangwa igihe cya challenge ntibyemewe.",
        )
    })?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(&challenge.address)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    let existing = sqlx::query(
        "SELECT a.id::text AS account_id, a.qr_reference FROM wallet_links w JOIN accounts a ON a.id = w.account_id WHERE w.network_passphrase = $1 AND w.stellar_address = $2",
    )
    .bind(TESTNET_PASSPHRASE)
    .bind(&challenge.address)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal_error)?;
    let (account_id, qr_reference, created) = if let Some(row) = existing {
        (
            row.try_get("account_id").map_err(internal_error)?,
            row.try_get("qr_reference").map_err(internal_error)?,
            false,
        )
    } else {
        let mut qr_bytes = [0_u8; 32];
        getrandom::getrandom(&mut qr_bytes)
            .map_err(|_| error(StatusCode::INTERNAL_SERVER_ERROR, "Randomness ntiboneka."))?;
        let qr_reference = URL_SAFE_NO_PAD.encode(qr_bytes);
        let account_id: String = sqlx::query_scalar(
            "INSERT INTO accounts (qr_reference) VALUES ($1) RETURNING id::text",
        )
        .bind(&qr_reference)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal_error)?;
        sqlx::query("INSERT INTO wallet_links (account_id, network_passphrase, stellar_address) VALUES ($1::uuid, $2, $3)")
            .bind(&account_id)
            .bind(TESTNET_PASSPHRASE)
            .bind(&challenge.address)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
        (account_id, qr_reference, true)
    };
    sqlx::query("UPDATE wallet_challenges SET consumed_at = now() WHERE id::text = $1")
        .bind(&input.challenge_id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    if created {
        sqlx::query("INSERT INTO security_events (account_id, challenge_id, event_kind) VALUES ($1::uuid, $2::uuid, 'account_created')")
            .bind(&account_id)
            .bind(&input.challenge_id)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }
    sqlx::query("INSERT INTO security_events (account_id, challenge_id, event_kind) VALUES ($1::uuid, $2::uuid, 'proof_accepted')")
        .bind(&account_id)
        .bind(&input.challenge_id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    tx.commit().await.map_err(internal_error)?;
    Ok(Json(ProofResponse {
        account_id,
        qr_reference,
        status: "wallet_key_verified",
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;
    let database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await?;
    if database != "ihuza_web3_testnet" {
        return Err("Unexpected database".into());
    }
    let app = Router::new()
        .route("/wallet/challenges", post(issue_challenge))
        .route("/wallet/proofs", post(verify_proof))
        .layer(DefaultBodyLimit::max(4096))
        .with_state(AppState { pool });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
