use axum::{
    extract::{DefaultBodyLimit, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ihuza_wallet_proof::{
    valid_stellar_address, verify_challenge, Challenge, MerchantReview, TESTNET_PASSPHRASE,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::{
    net::SocketAddr,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    reviewers: Option<ReviewerSet>,
}

#[derive(Clone)]
struct ReviewerSet {
    addresses: [String; 3],
    hash: String,
}

fn sha256_hex(value: &str) -> String {
    Sha256::digest(value.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn reviewer_set_from_config(value: &str) -> Result<ReviewerSet, &'static str> {
    let mut addresses: Vec<String> = value.split(',').map(str::trim).map(str::to_owned).collect();
    if addresses.len() != 3
        || addresses
            .iter()
            .any(|address| !valid_stellar_address(address))
    {
        return Err("IHUZA_REVIEWER_ADDRESSES must contain exactly three valid Stellar addresses");
    }
    addresses.sort();
    addresses.dedup();
    if addresses.len() != 3 {
        return Err("Reviewer addresses must be distinct");
    }
    let hash = sha256_hex(&addresses.join("\n"));
    Ok(ReviewerSet {
        addresses: addresses.try_into().map_err(|_| "Invalid reviewer set")?,
        hash,
    })
}

fn require_reviewer_set(state: &AppState) -> Result<&ReviewerSet, (StatusCode, Json<ErrorBody>)> {
    state.reviewers.as_ref().ok_or_else(|| {
        error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Reviewer configuration ntirashyirwaho.",
        )
    })
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
}

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<ErrorBody>)>;
const WEB_ORIGIN: &str = "http://127.0.0.1:5173";
const COOKIE_NAME: &str = "ihuza_session";

fn require_origin(headers: &HeaderMap) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    if headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        != Some(WEB_ORIGIN)
    {
        return Err(error(StatusCode::FORBIDDEN, "Origin ntiyemewe."));
    }
    Ok(())
}

fn session_hash(headers: &HeaderMap) -> Result<[u8; 32], (StatusCode, Json<ErrorBody>)> {
    let cookie = headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .find_map(|part| part.trim().strip_prefix(&format!("{COOKIE_NAME}=")))
        })
        .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "Session ntiyemewe."))?;
    let bytes = URL_SAFE_NO_PAD
        .decode(cookie)
        .map_err(|_| error(StatusCode::UNAUTHORIZED, "Session ntiyemewe."))?;
    if bytes.len() != 32 {
        return Err(error(StatusCode::UNAUTHORIZED, "Session ntiyemewe."));
    }
    Ok(Sha256::digest(&bytes).into())
}

fn set_cookie(value: &str, max_age: u32) -> Result<HeaderMap, (StatusCode, Json<ErrorBody>)> {
    let mut headers = HeaderMap::new();
    let cookie =
        format!("{COOKIE_NAME}={value}; HttpOnly; SameSite=Strict; Path=/api; Max-Age={max_age}");
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie)
            .map_err(|_| error(StatusCode::INTERNAL_SERVER_ERROR, "Cookie ntiyemewe."))?,
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(headers)
}

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
    headers: HeaderMap,
    Json(input): Json<ChallengeRequest>,
) -> ApiResult<ChallengeResponse> {
    require_origin(&headers)?;
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
    headers: HeaderMap,
    Json(input): Json<ProofRequest>,
) -> Result<(HeaderMap, Json<ProofResponse>), (StatusCode, Json<ErrorBody>)> {
    require_origin(&headers)?;
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
    let mut token = [0_u8; 32];
    getrandom::getrandom(&mut token)
        .map_err(|_| error(StatusCode::INTERNAL_SERVER_ERROR, "Randomness ntiboneka."))?;
    let token_hash: [u8; 32] = Sha256::digest(token).into();
    let session_id: String = sqlx::query_scalar(
        "INSERT INTO account_sessions (account_id, token_hash, expires_at) VALUES ($1::uuid, $2, now() + interval '12 hours') RETURNING id::text",
    )
    .bind(&account_id)
    .bind(token_hash.as_slice())
    .fetch_one(&mut *tx)
    .await
    .map_err(internal_error)?;
    sqlx::query("INSERT INTO session_events (session_id, event_kind) VALUES ($1::uuid, 'created')")
        .bind(&session_id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    tx.commit().await.map_err(internal_error)?;
    let cookie = set_cookie(&URL_SAFE_NO_PAD.encode(token), 12 * 60 * 60)?;
    Ok((
        cookie,
        Json(ProofResponse {
            account_id,
            qr_reference,
            status: "wallet_key_verified",
        }),
    ))
}

#[derive(Serialize)]
struct SessionResponse {
    account_id: String,
    qr_reference: String,
    stellar_address: String,
    expires_at: i64,
}

#[derive(Serialize)]
struct LogoutResponse {
    status: &'static str,
}

async fn read_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<SessionResponse>), (StatusCode, Json<ErrorBody>)> {
    let hash = session_hash(&headers)?;
    let row = sqlx::query(
        "SELECT a.id::text AS account_id, a.qr_reference, w.stellar_address, extract(epoch from s.expires_at)::bigint AS expires_at FROM account_sessions s JOIN accounts a ON a.id = s.account_id JOIN wallet_links w ON w.account_id = a.id WHERE s.token_hash = $1 AND s.revoked_at IS NULL AND s.expires_at > now() AND w.network_passphrase = $2",
    )
    .bind(hash.as_slice())
    .bind(TESTNET_PASSPHRASE)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "Session ntiyemewe."))?;
    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok((
        response_headers,
        Json(SessionResponse {
            account_id: row.try_get("account_id").map_err(internal_error)?,
            qr_reference: row.try_get("qr_reference").map_err(internal_error)?,
            stellar_address: row.try_get("stellar_address").map_err(internal_error)?,
            expires_at: row.try_get("expires_at").map_err(internal_error)?,
        }),
    ))
}

async fn session_account_id(
    pool: &PgPool,
    headers: &HeaderMap,
) -> Result<String, (StatusCode, Json<ErrorBody>)> {
    let hash = session_hash(headers)?;
    sqlx::query_scalar(
        "SELECT account_id::text FROM account_sessions WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > now()",
    )
    .bind(hash.as_slice())
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "Session ntiyemewe."))
}

async fn reviewer_session(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(String, String), (StatusCode, Json<ErrorBody>)> {
    let reviewers = require_reviewer_set(state)?;
    let hash = session_hash(headers)?;
    let row = sqlx::query(
        "SELECT s.account_id::text AS account_id, w.stellar_address FROM account_sessions s JOIN wallet_links w ON w.account_id = s.account_id WHERE s.token_hash = $1 AND s.revoked_at IS NULL AND s.expires_at > now() AND w.network_passphrase = $2",
    )
    .bind(hash.as_slice())
    .bind(TESTNET_PASSPHRASE)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "Session ntiyemewe."))?;
    let address: String = row.try_get("stellar_address").map_err(internal_error)?;
    if !reviewers.addresses.contains(&address) {
        return Err(error(StatusCode::FORBIDDEN, "Reviewer ntiyemewe."));
    }
    Ok((row.try_get("account_id").map_err(internal_error)?, address))
}

#[derive(Serialize)]
struct ReviewerStatusResponse {
    eligible: bool,
}

async fn reviewer_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<ReviewerStatusResponse>), (StatusCode, Json<ErrorBody>)> {
    let hash = session_hash(&headers)?;
    let address: String = sqlx::query_scalar(
        "SELECT w.stellar_address FROM account_sessions s JOIN wallet_links w ON w.account_id = s.account_id WHERE s.token_hash = $1 AND s.revoked_at IS NULL AND s.expires_at > now() AND w.network_passphrase = $2",
    )
    .bind(hash.as_slice())
    .bind(TESTNET_PASSPHRASE)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "Session ntiyemewe."))?;
    let eligible = state
        .reviewers
        .as_ref()
        .is_some_and(|reviewers| reviewers.addresses.contains(&address));
    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok((response_headers, Json(ReviewerStatusResponse { eligible })))
}

async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<LogoutResponse>), (StatusCode, Json<ErrorBody>)> {
    require_origin(&headers)?;
    let hash = session_hash(&headers)?;
    let mut tx = state.pool.begin().await.map_err(internal_error)?;
    let session_id: Option<String> = sqlx::query_scalar(
        "UPDATE account_sessions SET revoked_at = now() WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > now() RETURNING id::text",
    )
    .bind(hash.as_slice())
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal_error)?;
    let session_id =
        session_id.ok_or_else(|| error(StatusCode::UNAUTHORIZED, "Session ntiyemewe."))?;
    sqlx::query("INSERT INTO session_events (session_id, event_kind) VALUES ($1::uuid, 'revoked')")
        .bind(&session_id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    tx.commit().await.map_err(internal_error)?;
    Ok((
        set_cookie("", 0)?,
        Json(LogoutResponse {
            status: "signed_out",
        }),
    ))
}

#[derive(Serialize)]
struct QrResolution {
    account_kind: String,
    flow: &'static str,
}

fn qr_flow(account_kind: &str) -> &'static str {
    match account_kind {
        "person" => "person_transfer_preview",
        // Merchant approval, catalog and checkout are not implemented yet.
        _ => "unavailable",
    }
}

async fn resolve_qr(
    State(state): State<AppState>,
    Path(reference): Path<String>,
) -> Result<(HeaderMap, Json<QrResolution>), (StatusCode, Json<ErrorBody>)> {
    if reference.len() != 43
        || !reference
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(error(StatusCode::BAD_REQUEST, "QR reference ntiyemewe."));
    }
    let row = sqlx::query("SELECT a.id::text AS account_id, m.proposed_name FROM accounts a LEFT JOIN merchant_applications m ON m.account_id = a.id WHERE a.qr_reference = $1")
        .bind(&reference)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| error(StatusCode::NOT_FOUND, "Konti ntiyabonetse."))?;
    let account_id: String = row.try_get("account_id").map_err(internal_error)?;
    let proposed_name: Option<String> = row.try_get("proposed_name").map_err(internal_error)?;
    let account_kind = if let Some(name) = proposed_name {
        if approval_count(&state, &account_id, &name).await? >= 2 {
            "merchant"
        } else {
            "person"
        }
    } else {
        "person"
    }
    .to_owned();
    let mut headers = HeaderMap::new();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok((
        headers,
        Json(QrResolution {
            flow: qr_flow(&account_kind),
            account_kind,
        }),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MerchantApplicationRequest {
    proposed_name: String,
}

#[derive(Serialize)]
struct MerchantApplicationResponse {
    proposed_name: String,
    status: &'static str,
}

fn valid_merchant_name(name: &str) -> bool {
    let length = name.chars().count();
    (2..=80).contains(&length) && name.trim() == name && !name.chars().any(char::is_control)
}

fn valid_product_name(name: &str) -> bool {
    let length = name.chars().count();
    (2..=120).contains(&length) && name.trim() == name && !name.chars().any(char::is_control)
}

fn canonical_price(value: &str) -> Option<String> {
    let (whole, fraction) = value.split_once('.')?;
    if whole.is_empty()
        || whole.len() > 18
        || (whole.len() > 1 && whole.starts_with('0'))
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || fraction.is_empty()
        || fraction.len() > 2
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    let minor = match fraction.len() {
        1 => format!("{fraction}0"),
        2 => fraction.to_owned(),
        _ => return None,
    };
    if whole == "0" && minor == "00" {
        return None;
    }
    Some(format!("{whole}.{minor}"))
}

async fn approval_count(
    state: &AppState,
    account_id: &str,
    proposed_name: &str,
) -> Result<usize, (StatusCode, Json<ErrorBody>)> {
    let Some(reviewers) = &state.reviewers else {
        return Ok(0);
    };
    let rows = sqlx::query(
        "SELECT reviewer_address, name_hash, nonce_hex, signature_base64, extract(epoch from issued_at)::bigint AS issued_at, extract(epoch from expires_at)::bigint AS expires_at, extract(epoch from approved_at)::bigint AS approved_at FROM merchant_approval_votes WHERE account_id = $1::uuid AND reviewer_set_hash = $2",
    )
    .bind(account_id)
    .bind(&reviewers.hash)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;
    let expected_name_hash = sha256_hex(proposed_name);
    let mut count = 0;
    for row in rows {
        let address: String = row.try_get("reviewer_address").map_err(internal_error)?;
        let name_hash: String = row.try_get("name_hash").map_err(internal_error)?;
        if !reviewers.addresses.contains(&address) || name_hash != expected_name_hash {
            continue;
        }
        let issued_at: i64 = row.try_get("issued_at").map_err(internal_error)?;
        let expires_at: i64 = row.try_get("expires_at").map_err(internal_error)?;
        let approved_at: i64 = row.try_get("approved_at").map_err(internal_error)?;
        let (Ok(issued_at), Ok(expires_at), Ok(approved_at)) = (
            issued_at.try_into(),
            expires_at.try_into(),
            approved_at.try_into(),
        ) else {
            continue;
        };
        let review = MerchantReview {
            reviewer_address: address,
            account_id: account_id.to_owned(),
            name_hash,
            reviewer_set_hash: reviewers.hash.clone(),
            nonce_hex: row.try_get("nonce_hex").map_err(internal_error)?,
            issued_at,
            expires_at,
        };
        let signature: String = row.try_get("signature_base64").map_err(internal_error)?;
        if review.verify(approved_at, &signature) {
            count += 1;
        }
    }
    Ok(count)
}

#[derive(Serialize)]
struct ReviewMessageResponse {
    proposed_name: String,
    nonce_hex: String,
    issued_at: u64,
    expires_at: u64,
    message: String,
}

async fn review_target(
    state: &AppState,
    reference: &str,
    reviewer_account_id: &str,
) -> Result<(String, String), (StatusCode, Json<ErrorBody>)> {
    if reference.len() != 43
        || !reference
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(error(StatusCode::BAD_REQUEST, "QR reference ntiyemewe."));
    }
    let row = sqlx::query(
        "SELECT a.id::text AS account_id, m.proposed_name FROM accounts a JOIN merchant_applications m ON m.account_id = a.id WHERE a.qr_reference = $1",
    )
    .bind(reference)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| error(StatusCode::NOT_FOUND, "Ubusabe ntibuboneka."))?;
    let account_id: String = row.try_get("account_id").map_err(internal_error)?;
    if account_id == reviewer_account_id {
        return Err(error(StatusCode::FORBIDDEN, "Kwiyemeza ntibyemewe."));
    }
    Ok((
        account_id,
        row.try_get("proposed_name").map_err(internal_error)?,
    ))
}

async fn review_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(reference): Path<String>,
) -> Result<(HeaderMap, Json<ReviewMessageResponse>), (StatusCode, Json<ErrorBody>)> {
    let reviewers = require_reviewer_set(&state)?;
    let (reviewer_account_id, reviewer_address) = reviewer_session(&state, &headers).await?;
    let (account_id, proposed_name) =
        review_target(&state, &reference, &reviewer_account_id).await?;
    let issued_at = unix_now()?;
    let mut nonce = [0_u8; 32];
    getrandom::getrandom(&mut nonce)
        .map_err(|_| error(StatusCode::INTERNAL_SERVER_ERROR, "Randomness ntiboneka."))?;
    let review = MerchantReview {
        reviewer_address,
        account_id,
        name_hash: sha256_hex(&proposed_name),
        reviewer_set_hash: reviewers.hash.clone(),
        nonce_hex: nonce.iter().map(|byte| format!("{byte:02x}")).collect(),
        issued_at,
        expires_at: issued_at + 300,
    };
    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok((
        response_headers,
        Json(ReviewMessageResponse {
            proposed_name,
            nonce_hex: review.nonce_hex.clone(),
            issued_at,
            expires_at: review.expires_at,
            message: review.message(),
        }),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewProofRequest {
    nonce_hex: String,
    issued_at: u64,
    expires_at: u64,
    signature_base64: String,
}

#[derive(Serialize)]
struct ReviewProofResponse {
    votes: usize,
    threshold: usize,
    approved: bool,
}

async fn submit_review(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(reference): Path<String>,
    Json(input): Json<ReviewProofRequest>,
) -> Result<(HeaderMap, Json<ReviewProofResponse>), (StatusCode, Json<ErrorBody>)> {
    require_origin(&headers)?;
    let reviewers = require_reviewer_set(&state)?;
    if input.signature_base64.len() > 200 {
        return Err(error(StatusCode::BAD_REQUEST, "Umukono ntiwemewe."));
    }
    let (reviewer_account_id, reviewer_address) = reviewer_session(&state, &headers).await?;
    let (account_id, proposed_name) =
        review_target(&state, &reference, &reviewer_account_id).await?;
    let review = MerchantReview {
        reviewer_address: reviewer_address.clone(),
        account_id: account_id.clone(),
        name_hash: sha256_hex(&proposed_name),
        reviewer_set_hash: reviewers.hash.clone(),
        nonce_hex: input.nonce_hex,
        issued_at: input.issued_at,
        expires_at: input.expires_at,
    };
    if !review.verify(unix_now()?, &input.signature_base64) {
        return Err(error(
            StatusCode::UNAUTHORIZED,
            "Umukono wa reviewer ntiwemewe.",
        ));
    }
    let inserted: Option<String> = sqlx::query_scalar(
        "INSERT INTO merchant_approval_votes (account_id, reviewer_address, reviewer_set_hash, name_hash, nonce_hex, issued_at, expires_at, signature_base64) VALUES ($1::uuid, $2, $3, $4, $5, to_timestamp($6::double precision), to_timestamp($7::double precision), $8) ON CONFLICT DO NOTHING RETURNING id::text",
    )
    .bind(&account_id)
    .bind(&reviewer_address)
    .bind(&review.reviewer_set_hash)
    .bind(&review.name_hash)
    .bind(&review.nonce_hex)
    .bind(review.issued_at as i64)
    .bind(review.expires_at as i64)
    .bind(&input.signature_base64)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;
    if inserted.is_none() {
        return Err(error(StatusCode::CONFLICT, "Vote yamaze kubikwa."));
    }
    let votes = approval_count(&state, &account_id, &proposed_name).await?;
    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok((
        response_headers,
        Json(ReviewProofResponse {
            votes,
            threshold: 2,
            approved: votes >= 2,
        }),
    ))
}

async fn get_merchant_application(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<MerchantApplicationResponse>), (StatusCode, Json<ErrorBody>)> {
    let account_id = session_account_id(&state.pool, &headers).await?;
    let proposed_name: String = sqlx::query_scalar(
        "SELECT proposed_name FROM merchant_applications WHERE account_id = $1::uuid",
    )
    .bind(&account_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| error(StatusCode::NOT_FOUND, "Ubusabe ntibuboneka."))?;
    let status = if approval_count(&state, &account_id, &proposed_name).await? >= 2 {
        "approved"
    } else {
        "pending"
    };
    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok((
        response_headers,
        Json(MerchantApplicationResponse {
            proposed_name,
            status,
        }),
    ))
}

async fn submit_merchant_application(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<MerchantApplicationRequest>,
) -> Result<(StatusCode, HeaderMap, Json<MerchantApplicationResponse>), (StatusCode, Json<ErrorBody>)>
{
    require_origin(&headers)?;
    if !valid_merchant_name(&input.proposed_name) {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "Izina ry'ubucuruzi ntiryemewe.",
        ));
    }
    let account_id = session_account_id(&state.pool, &headers).await?;
    let inserted: Option<String> = sqlx::query_scalar(
        "INSERT INTO merchant_applications (account_id, proposed_name) VALUES ($1::uuid, $2) ON CONFLICT (account_id) DO NOTHING RETURNING proposed_name",
    )
    .bind(&account_id)
    .bind(&input.proposed_name)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;
    let (status, proposed_name) = if let Some(name) = inserted {
        (StatusCode::CREATED, name)
    } else {
        let existing: String = sqlx::query_scalar(
            "SELECT proposed_name FROM merchant_applications WHERE account_id = $1::uuid",
        )
        .bind(&account_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?;
        if existing != input.proposed_name {
            return Err(error(StatusCode::CONFLICT, "Ubusabe busanzwe buriho."));
        }
        (StatusCode::OK, existing)
    };
    let application_status = if approval_count(&state, &account_id, &proposed_name).await? >= 2 {
        "approved"
    } else {
        "pending"
    };
    Ok((
        status,
        {
            let mut headers = HeaderMap::new();
            headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
            headers
        },
        Json(MerchantApplicationResponse {
            proposed_name,
            status: application_status,
        }),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateProductRequest {
    name: String,
    unit_price: String,
}

#[derive(Serialize)]
struct ProductResponse {
    id: String,
    name: String,
    unit_price: String,
    currency_code: &'static str,
}

async fn approved_merchant(
    state: &AppState,
    account_id: &str,
) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    let proposed_name: Option<String> = sqlx::query_scalar(
        "SELECT proposed_name FROM merchant_applications WHERE account_id = $1::uuid",
    )
    .bind(account_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;
    let Some(proposed_name) = proposed_name else {
        return Err(error(StatusCode::FORBIDDEN, "Merchant ntiyemejwe."));
    };
    if approval_count(state, account_id, &proposed_name).await? < 2 {
        return Err(error(StatusCode::FORBIDDEN, "Merchant ntiyemejwe."));
    }
    Ok(())
}

async fn create_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CreateProductRequest>,
) -> Result<(StatusCode, HeaderMap, Json<ProductResponse>), (StatusCode, Json<ErrorBody>)> {
    require_origin(&headers)?;
    if !valid_product_name(&input.name) {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "Izina ry'igicuruzwa ntiryemewe.",
        ));
    }
    let unit_price = canonical_price(&input.unit_price)
        .ok_or_else(|| error(StatusCode::BAD_REQUEST, "Igiciro nticyemewe."))?;
    let account_id = session_account_id(&state.pool, &headers).await?;
    approved_merchant(&state, &account_id).await?;
    let row = sqlx::query(
        "INSERT INTO merchant_products (merchant_account_id, name, unit_price) VALUES ($1::uuid, $2, $3::numeric) RETURNING id::text AS id, name, unit_price::text AS unit_price",
    )
    .bind(&account_id)
    .bind(&input.name)
    .bind(&unit_price)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;
    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok((
        StatusCode::CREATED,
        response_headers,
        Json(ProductResponse {
            id: row.try_get("id").map_err(internal_error)?,
            name: row.try_get("name").map_err(internal_error)?,
            unit_price: row.try_get("unit_price").map_err(internal_error)?,
            currency_code: "RWF",
        }),
    ))
}

async fn list_products(
    State(state): State<AppState>,
    Path(reference): Path<String>,
) -> Result<(HeaderMap, Json<Vec<ProductResponse>>), (StatusCode, Json<ErrorBody>)> {
    if reference.len() != 43
        || !reference
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(error(StatusCode::BAD_REQUEST, "QR reference ntiyemewe."));
    }
    let account_id: String =
        sqlx::query_scalar("SELECT id::text FROM accounts WHERE qr_reference = $1")
            .bind(&reference)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?
            .ok_or_else(|| error(StatusCode::NOT_FOUND, "Konti ntiyabonetse."))?;
    approved_merchant(&state, &account_id).await?;
    let rows = sqlx::query(
        "SELECT id::text AS id, name, unit_price::text AS unit_price FROM merchant_products WHERE merchant_account_id = $1::uuid ORDER BY created_at, id LIMIT 100",
    )
    .bind(&account_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;
    let products = rows
        .into_iter()
        .map(|row| {
            Ok(ProductResponse {
                id: row.try_get("id").map_err(internal_error)?,
                name: row.try_get("name").map_err(internal_error)?,
                unit_price: row.try_get("unit_price").map_err(internal_error)?,
                currency_code: "RWF",
            })
        })
        .collect::<Result<Vec<_>, (StatusCode, Json<ErrorBody>)>>()?;
    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok((response_headers, Json(products)))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let reviewers = match std::env::var("IHUZA_REVIEWER_ADDRESSES") {
        Ok(value) => Some(reviewer_set_from_config(&value)?),
        Err(std::env::VarError::NotPresent) => None,
        Err(error) => return Err(error.into()),
    };
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
        .route("/api/wallet/challenges", post(issue_challenge))
        .route("/api/wallet/proofs", post(verify_proof))
        .route("/api/session", get(read_session))
        .route("/api/reviewer-status", get(reviewer_status))
        .route("/api/session/logout", post(logout))
        .route("/api/qr/{reference}", get(resolve_qr))
        .route(
            "/api/merchant-review/{reference}",
            get(review_message).post(submit_review),
        )
        .route(
            "/api/merchant-application",
            get(get_merchant_application).post(submit_merchant_application),
        )
        .route("/api/merchant-products", post(create_product))
        .route("/api/merchant-products/{reference}", get(list_products))
        .layer(DefaultBodyLimit::max(4096))
        .with_state(AppState { pool, reviewers });
    let bind = std::env::var("IHUZA_API_BIND").unwrap_or_else(|_| "127.0.0.1:3000".to_owned());
    let address: SocketAddr = bind.parse()?;
    if address.ip().to_string() != "127.0.0.1" {
        return Err("API bind must be 127.0.0.1".into());
    }
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{canonical_price, qr_flow, valid_merchant_name, valid_product_name};

    #[test]
    fn merchant_qr_never_opens_checkout_without_approval() {
        assert_eq!(qr_flow("person"), "person_transfer_preview");
        assert_eq!(qr_flow("merchant"), "unavailable");
    }

    #[test]
    fn merchant_name_rejects_controls_and_padding() {
        assert!(valid_merchant_name("Iduka Ryawe"));
        assert!(!valid_merchant_name(" A "));
        assert!(!valid_merchant_name("A\nB"));
        assert!(!valid_merchant_name("A"));
    }

    #[test]
    fn product_price_is_decimal_and_positive() {
        assert!(valid_product_name("Umuceri"));
        assert!(!valid_product_name(" Umuceri "));
        assert_eq!(canonical_price("1000.5"), Some("1000.50".to_owned()));
        assert_eq!(canonical_price("0.01"), Some("0.01".to_owned()));
        for price in ["0.00", "01.00", "1", "1.234", "-1.00", "1e3", "1,000.00"] {
            assert!(canonical_price(price).is_none(), "{price}");
        }
    }
}
