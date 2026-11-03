use std::{
    collections::{HashMap, HashSet},
    env,
};

use argon2::{
    Argon2, PasswordHasher, PasswordVerifier,
    password_hash::{PasswordHash, SaltString},
};
use axum::{
    Router,
    extract::{Json, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, patch, post},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{PgConnection, PgPool, postgres::PgPoolOptions};
use tokio::net::TcpListener;

const MAX_NAME_LENGTH: usize = 100;
const MAX_PHONE_LENGTH: usize = 32;
const MAX_EMAIL_LENGTH: usize = 254;
const MIN_PASSWORD_LENGTH: usize = 12;
const SESSION_TOKEN_BYTES: usize = 32;
const SESSION_DURATION_SECONDS: i64 = 60 * 60 * 24 * 30;
const MAX_FAILED_LOGIN_ATTEMPTS: i32 = 5;
const LOGIN_FAILURE_WINDOW_SECONDS: i64 = 60 * 15;
const LOGIN_LOCKOUT_SECONDS: i64 = 60 * 15;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

#[derive(Deserialize)]
struct RegisterRequest {
    phone_number: String,
    email: Option<String>,
    password: String,
    first_name: String,
    last_name: Option<String>,
}

#[derive(Deserialize)]
struct LoginRequest {
    identifier: String,
    password: String,
}

#[derive(Deserialize)]
struct PasswordChangeRequest {
    current_password: String,
    new_password: String,
}

#[derive(Deserialize)]
struct MerchantApplicationRequest {
    business_name: String,
    business_type: Option<String>,
    phone_number: Option<String>,
    address: Option<String>,
}

#[derive(Serialize)]
struct MerchantApplicationResponse {
    id: String,
    status: &'static str,
}

#[derive(Deserialize)]
struct CreateProductRequest {
    name: String,
    description: Option<String>,
    price: String,
    sku: Option<String>,
    barcode: Option<String>,
}

#[derive(Deserialize)]
struct UpdateProductPriceRequest {
    price: String,
}

#[derive(Serialize, Deserialize)]
struct CreatedProductResponse {
    id: String,
    name: String,
    price: String,
    currency: String,
}

#[derive(Deserialize)]
struct ProductListQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Serialize, Deserialize)]
struct CatalogProductResponse {
    id: String,
    name: String,
    price: String,
    currency: String,
    image_url: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct MerchantCatalogResponse {
    business_name: String,
    products: Vec<CatalogProductResponse>,
}

#[derive(Deserialize)]
struct CheckoutQuoteRequest {
    items: Vec<CheckoutQuoteItemRequest>,
}

#[derive(Deserialize, Serialize)]
struct CheckoutQuoteItemRequest {
    product_id: String,
    quantity: String,
}

#[derive(Serialize, Deserialize)]
struct CheckoutQuoteLineResponse {
    product_id: String,
    product_name: String,
    quantity: String,
    unit_price: String,
    subtotal: String,
}

#[derive(Serialize, Deserialize)]
struct CheckoutQuoteResponse {
    business_name: String,
    currency: String,
    items: Vec<CheckoutQuoteLineResponse>,
    total: String,
}

#[derive(Deserialize)]
struct CreateInvoiceDraftRequest {
    items: Vec<CheckoutQuoteItemRequest>,
    expected_total: String,
    idempotency_key: String,
}

#[derive(Serialize, Deserialize)]
struct InvoiceDraftResponse {
    id: String,
    invoice_number: String,
    total: String,
    currency: String,
    payment_status: String,
}

#[derive(Serialize, Deserialize)]
struct InvoiceSummaryResponse {
    id: String,
    invoice_number: String,
    business_name: String,
    total: String,
    currency: String,
    payment_status: String,
    invoice_date: String,
}

#[derive(Serialize, Deserialize)]
struct InvoiceItemResponse {
    product_id: Option<String>,
    product_name: String,
    quantity: String,
    unit_price: String,
    subtotal: String,
}

#[derive(Serialize, Deserialize)]
struct InvoiceDetailResponse {
    #[serde(flatten)]
    invoice: InvoiceSummaryResponse,
    items: Vec<InvoiceItemResponse>,
}

#[derive(Deserialize, Serialize)]
struct LoginResponse {
    access_token: String,
    token_type: String,
    expires_in_seconds: i64,
}

#[derive(Deserialize, Serialize)]
struct CurrentUserResponse {
    id: String,
    phone_number: String,
    email: Option<String>,
    first_name: String,
    last_name: Option<String>,
    roles: Vec<String>,
}

#[derive(Deserialize, Serialize)]
struct AccountQrReferenceResponse {
    reference: String,
}

#[derive(Serialize)]
struct AccountQrDestinationResponse {
    account_type: &'static str,
    display_name: String,
    currency: &'static str,
}

#[derive(Deserialize, Serialize)]
struct WalletSummaryResponse {
    currency: String,
    balance: String,
    status: String,
    money_operations_enabled: bool,
}

#[derive(Debug)]
struct NewRegistration {
    phone_number: String,
    email: Option<String>,
    password: String,
    first_name: String,
    last_name: Option<String>,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: &'static str,
}

impl ApiError {
    fn bad_request(message: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message,
        }
    }

    fn conflict(message: &'static str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message,
        }
    }

    fn forbidden() -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: "Admin access is required",
        }
    }

    fn merchant_forbidden() -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: "Approved merchant access is required",
        }
    }

    fn customer_forbidden() -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: "Customer access is required",
        }
    }

    fn not_found(message: &'static str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message,
        }
    }

    fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: "Invalid credentials",
        }
    }

    fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "An internal server error occurred",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, self.message).into_response()
    }
}

async fn health() -> &'static str {
    "IHUZA+ is healthy"
}

fn app(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(current_user))
        .route("/account/qr-reference", get(account_qr_reference))
        .route("/wallet", post(open_wallet).get(get_wallet))
        .route("/auth/password", patch(change_password))
        .route("/merchants/apply", post(apply_for_merchant))
        .route(
            "/merchant/products",
            post(create_product).get(list_merchant_products),
        )
        .route("/merchant/products/{id}", patch(update_product_price))
        .route("/qr/{reference}", get(resolve_account_qr))
        .route("/qr/{reference}/catalog", get(catalog_from_account_qr))
        .route("/qr/{reference}/quote", post(quote_from_account_qr))
        .route("/qr/{reference}/invoices", post(create_invoice_draft))
        .route("/invoices", get(list_invoices))
        .route("/invoices/{id}", get(get_invoice))
        .route(
            "/admin/merchant-applications/{id}/approve",
            post(approve_merchant_application),
        )
        .with_state(AppState { pool })
}

async fn authenticated_merchant_id(pool: &PgPool, headers: &HeaderMap) -> Result<String, ApiError> {
    let Some(session_token) = bearer_token(headers) else {
        return Err(ApiError::unauthorized());
    };

    let account = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT u.id::text, m.id::text
         FROM user_sessions s
         JOIN users u ON u.id = s.user_id
         LEFT JOIN merchant_profiles m ON m.user_id = u.id
           AND m.status = 'APPROVED'
           AND EXISTS (
               SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id
               WHERE ur.user_id = u.id AND r.name = 'MERCHANT'
           )
         WHERE s.token_hash = $1
           AND s.revoked_at IS NULL
           AND s.expires_at > NOW()
           AND u.is_active = TRUE",
    )
    .bind(hash_session_token(session_token))
    .fetch_optional(pool)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(ApiError::unauthorized)?;
    account.1.ok_or_else(ApiError::merchant_forbidden)
}

async fn create_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateProductRequest>,
) -> Result<(StatusCode, Json<CreatedProductResponse>), ApiError> {
    let merchant_id = authenticated_merchant_id(&state.pool, &headers).await?;

    let name = request.name.trim();
    let price = request.price.trim();
    let description = optional_trimmed(request.description);
    let sku = optional_trimmed(request.sku);
    let barcode = optional_trimmed(request.barcode);
    if name.is_empty() || name.chars().count() > 150 {
        return Err(ApiError::bad_request("A valid product name is required"));
    }
    if description
        .as_ref()
        .is_some_and(|value| value.chars().count() > 2_000)
        || sku.as_ref().is_some_and(|value| value.chars().count() > 64)
        || barcode
            .as_ref()
            .is_some_and(|value| value.chars().count() > 64)
    {
        return Err(ApiError::bad_request("Product details are too long"));
    }
    if !valid_catalog_price(price) {
        return Err(ApiError::bad_request(
            "Price must be a non-negative decimal with at most two decimal places",
        ));
    }

    let created = sqlx::query_as::<_, (String, String, String, String)>(
        "INSERT INTO products (merchant_id, name, description, price, sku, barcode)
         VALUES ($1::uuid, $2, $3, $4::numeric, $5, $6)
         RETURNING id::text, name, price::text, btrim(currency)",
    )
    .bind(&merchant_id)
    .bind(name)
    .bind(&description)
    .bind(price)
    .bind(&sku)
    .bind(&barcode)
    .fetch_one(&state.pool)
    .await;
    let (id, name, price, currency) = match created {
        Ok(created) => created,
        Err(error) if is_unique_violation(&error) => {
            return Err(ApiError::conflict(
                "SKU or barcode is already used by this merchant",
            ));
        }
        Err(_) => return Err(ApiError::internal()),
    };

    Ok((
        StatusCode::CREATED,
        Json(CreatedProductResponse {
            id,
            name,
            price,
            currency,
        }),
    ))
}

async fn update_product_price(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<UpdateProductPriceRequest>,
) -> Result<Json<CreatedProductResponse>, ApiError> {
    let merchant_id = authenticated_merchant_id(&state.pool, &headers).await?;
    let price = request.price.trim();
    if !valid_catalog_price(price) {
        return Err(ApiError::bad_request(
            "Price must be a non-negative decimal with at most two decimal places",
        ));
    }

    let (id, name, price, currency) = sqlx::query_as::<_, (String, String, String, String)>(
        "UPDATE products SET price = $3::numeric, updated_at = NOW()
         WHERE merchant_id = $1::uuid AND id::text = $2 AND is_active = TRUE
         RETURNING id::text, name, price::text, btrim(currency)",
    )
    .bind(&merchant_id)
    .bind(&product_id)
    .bind(price)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(|| ApiError::not_found("Product was not found"))?;

    Ok(Json(CreatedProductResponse {
        id,
        name,
        price,
        currency,
    }))
}

async fn list_merchant_products(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ProductListQuery>,
) -> Result<Json<Vec<CreatedProductResponse>>, ApiError> {
    let merchant_id = authenticated_merchant_id(&state.pool, &headers).await?;
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    if !(1..=50).contains(&limit) || !(0..=100_000).contains(&offset) {
        return Err(ApiError::bad_request("Invalid product page"));
    }

    let products = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT id::text, name, price::text, btrim(currency)
         FROM products
         WHERE merchant_id = $1::uuid AND is_active = TRUE
         ORDER BY created_at DESC, id DESC
         LIMIT $2 OFFSET $3",
    )
    .bind(&merchant_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;

    Ok(Json(
        products
            .into_iter()
            .map(|(id, name, price, currency)| CreatedProductResponse {
                id,
                name,
                price,
                currency,
            })
            .collect(),
    ))
}

async fn catalog_from_account_qr(
    State(state): State<AppState>,
    Path(reference): Path<String>,
    headers: HeaderMap,
    Query(query): Query<ProductListQuery>,
) -> Result<Json<MerchantCatalogResponse>, ApiError> {
    require_customer(&state.pool, &headers).await?;
    let merchant = approved_merchant_from_reference(&state.pool, &reference).await?;

    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    if !(1..=50).contains(&limit) || !(0..=100_000).contains(&offset) {
        return Err(ApiError::bad_request("Invalid product page"));
    }

    let products = sqlx::query_as::<_, (String, String, String, String, Option<String>)>(
        "SELECT p.id::text, p.name, p.price::text, btrim(p.currency), image.image_url
         FROM products p
         LEFT JOIN LATERAL (
             SELECT pi.image_url FROM product_images pi
             WHERE pi.product_id = p.id AND pi.is_primary = TRUE
             LIMIT 1
         ) image ON TRUE
         WHERE p.merchant_id = $1::uuid AND p.is_active = TRUE
         ORDER BY p.created_at DESC, p.id DESC
         LIMIT $2 OFFSET $3",
    )
    .bind(&merchant.0)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;

    Ok(Json(MerchantCatalogResponse {
        business_name: merchant.1,
        products: products
            .into_iter()
            .map(
                |(id, name, price, currency, image_url)| CatalogProductResponse {
                    id,
                    name,
                    price,
                    currency,
                    image_url,
                },
            )
            .collect(),
    }))
}

async fn require_customer(pool: &PgPool, headers: &HeaderMap) -> Result<String, ApiError> {
    let Some(session_token) = bearer_token(&headers) else {
        return Err(ApiError::unauthorized());
    };
    let (customer_id, is_customer) = sqlx::query_as::<_, (String, bool)>(
        "SELECT u.id::text, EXISTS (
             SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id
             WHERE ur.user_id = u.id AND r.name = 'CUSTOMER'
         )
         FROM user_sessions s JOIN users u ON u.id = s.user_id
         WHERE s.token_hash = $1
           AND s.revoked_at IS NULL
           AND s.expires_at > NOW()
           AND u.is_active = TRUE",
    )
    .bind(hash_session_token(session_token))
    .fetch_optional(pool)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(ApiError::unauthorized)?;
    if !is_customer {
        return Err(ApiError::customer_forbidden());
    }
    Ok(customer_id)
}

async fn wallet_summary(
    connection: &mut PgConnection,
    user_id: &str,
) -> Result<WalletSummaryResponse, ApiError> {
    let (currency, balance, status) = sqlx::query_as::<_, (String, String, String)>(
        "SELECT b.currency, b.balance::text, a.status
         FROM wallet_accounts a
         JOIN wallet_balances b ON b.account_id = a.id
         JOIN users u ON u.id = a.user_id AND u.is_active = TRUE
         WHERE a.user_id = $1::uuid
           AND EXISTS (
               SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id
               WHERE ur.user_id = u.id AND r.name = 'CUSTOMER'
           )",
    )
    .bind(user_id)
    .fetch_optional(connection)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(|| ApiError::not_found("Wallet has not been opened"))?;
    Ok(WalletSummaryResponse {
        currency,
        balance,
        status,
        money_operations_enabled: false,
    })
}

async fn provision_wallet(
    connection: &mut PgConnection,
    user_id: &str,
) -> Result<WalletSummaryResponse, ApiError> {
    sqlx::query(
        "INSERT INTO wallet_accounts (user_id)
         SELECT u.id FROM users u
         WHERE u.id = $1::uuid AND u.is_active = TRUE
           AND EXISTS (
               SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id
               WHERE ur.user_id = u.id AND r.name = 'CUSTOMER'
           )
         ON CONFLICT (user_id) DO NOTHING",
    )
    .bind(user_id)
    .execute(&mut *connection)
    .await
    .map_err(|_| ApiError::internal())?;
    wallet_summary(connection, user_id).await
}

async fn open_wallet(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<WalletSummaryResponse>, ApiError> {
    let user_id = require_customer(&state.pool, &headers).await?;
    let mut tx = state.pool.begin().await.map_err(|_| ApiError::internal())?;
    let summary = provision_wallet(&mut tx, &user_id).await?;
    tx.commit().await.map_err(|_| ApiError::internal())?;
    Ok(Json(summary))
}

async fn get_wallet(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<WalletSummaryResponse>, ApiError> {
    let user_id = require_customer(&state.pool, &headers).await?;
    let mut connection = state
        .pool
        .acquire()
        .await
        .map_err(|_| ApiError::internal())?;
    Ok(Json(wallet_summary(&mut connection, &user_id).await?))
}

async fn approved_merchant_from_reference(
    pool: &PgPool,
    reference: &str,
) -> Result<(String, String), ApiError> {
    if reference.len() != 48 || !reference.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ApiError::not_found("Merchant catalog was not found"));
    }

    sqlx::query_as::<_, (String, String)>(
        "SELECT m.id::text, m.business_name
         FROM users u
         JOIN merchant_profiles m ON m.user_id = u.id AND m.status = 'APPROVED'
         JOIN user_roles ur ON ur.user_id = u.id
         JOIN roles r ON r.id = ur.role_id AND r.name = 'MERCHANT'
         WHERE u.account_qr_reference = $1 AND u.is_active = TRUE",
    )
    .bind(&reference)
    .fetch_optional(pool)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(|| ApiError::not_found("Merchant catalog was not found"))
}

async fn resolve_account_qr(
    State(state): State<AppState>,
    Path(reference): Path<String>,
    headers: HeaderMap,
) -> Result<Json<AccountQrDestinationResponse>, ApiError> {
    let viewer_id = require_customer(&state.pool, &headers).await?;
    if reference.len() != 48 || !reference.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ApiError::not_found("Account QR was not found"));
    }

    let destination = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, bool)>(
        "SELECT u.id::text, u.first_name, u.last_name, m.business_name,
                EXISTS (
                    SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id
                    WHERE ur.user_id = u.id AND r.name = 'CUSTOMER'
                )
         FROM users u
         LEFT JOIN merchant_profiles m ON m.user_id = u.id AND m.status = 'APPROVED'
           AND EXISTS (
               SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id
               WHERE ur.user_id = u.id AND r.name = 'MERCHANT'
           )
         WHERE u.account_qr_reference = $1 AND u.is_active = TRUE",
    )
    .bind(&reference)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(|| ApiError::not_found("Account QR was not found"))?;

    if destination.0 == viewer_id {
        return Err(ApiError::bad_request(
            "Your own QR is not a payment destination",
        ));
    }
    if let Some(business_name) = destination.3 {
        return Ok(Json(AccountQrDestinationResponse {
            account_type: "MERCHANT_CHECKOUT",
            display_name: business_name,
            currency: "RWF",
        }));
    }
    if !destination.4 {
        return Err(ApiError::not_found("Account QR was not found"));
    }
    let display_name = match destination.2 {
        Some(last_name) => format!("{} {}", destination.1, last_name),
        None => destination.1,
    };
    Ok(Json(AccountQrDestinationResponse {
        account_type: "PERSON_TRANSFER",
        display_name,
        currency: "RWF",
    }))
}

async fn quote_from_account_qr(
    State(state): State<AppState>,
    Path(reference): Path<String>,
    headers: HeaderMap,
    Json(request): Json<CheckoutQuoteRequest>,
) -> Result<Json<CheckoutQuoteResponse>, ApiError> {
    require_customer(&state.pool, &headers).await?;
    let merchant = approved_merchant_from_reference(&state.pool, &reference).await?;
    let mut connection = state
        .pool
        .acquire()
        .await
        .map_err(|_| ApiError::internal())?;
    Ok(Json(build_quote(&mut connection, merchant, request).await?))
}

async fn build_quote(
    connection: &mut sqlx::PgConnection,
    merchant: (String, String),
    request: CheckoutQuoteRequest,
) -> Result<CheckoutQuoteResponse, ApiError> {
    if request.items.is_empty() || request.items.len() > 50 {
        return Err(ApiError::bad_request("Quote must contain 1 to 50 items"));
    }

    let mut seen = HashSet::new();
    let mut selected = Vec::with_capacity(request.items.len());
    for item in request.items {
        let id = item.product_id.trim().to_ascii_lowercase();
        let quantity = parse_decimal_units(item.quantity.trim(), 17, 3)
            .filter(|quantity| *quantity > 0)
            .ok_or_else(|| ApiError::bad_request("Invalid item quantity"))?;
        if id.len() != 36
            || !id
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() || byte == b'-')
            || !seen.insert(id.clone())
        {
            return Err(ApiError::bad_request("Invalid or duplicate product ID"));
        }
        selected.push((id, quantity));
    }

    let ids: Vec<String> = selected.iter().map(|(id, _)| id.clone()).collect();
    let products = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT id::text, name, price::text, btrim(currency)
         FROM products
         WHERE merchant_id = $1::uuid AND id::text = ANY($2) AND is_active = TRUE
         FOR SHARE",
    )
    .bind(&merchant.0)
    .bind(&ids)
    .fetch_all(&mut *connection)
    .await
    .map_err(|_| ApiError::internal())?;
    let products: HashMap<_, _> = products
        .into_iter()
        .map(|(id, name, price, currency)| (id, (name, price, currency)))
        .collect();

    let mut lines = Vec::with_capacity(selected.len());
    let mut total_cents = 0_u128;
    const MAX_CENTS: u128 = 99_999_999_999_999_999_999;
    for (id, quantity_milli) in selected {
        let (name, price, currency) = products
            .get(&id)
            .ok_or_else(|| ApiError::bad_request("Product is unavailable for this merchant"))?;
        if currency != "RWF" {
            return Err(ApiError::bad_request("Unsupported product currency"));
        }
        let price_cents = parse_decimal_units(price, 18, 2).ok_or_else(ApiError::internal)?;
        let subtotal_cents = line_subtotal_cents(price_cents, quantity_milli)
            .filter(|amount| *amount <= MAX_CENTS)
            .ok_or_else(|| ApiError::bad_request("Quote amount is too large"))?;
        total_cents = total_cents
            .checked_add(subtotal_cents)
            .filter(|amount| *amount <= MAX_CENTS)
            .ok_or_else(|| ApiError::bad_request("Quote amount is too large"))?;
        lines.push(CheckoutQuoteLineResponse {
            product_id: id,
            product_name: name.clone(),
            quantity: format!("{}.{:03}", quantity_milli / 1000, quantity_milli % 1000),
            unit_price: format_money(price_cents),
            subtotal: format_money(subtotal_cents),
        });
    }

    Ok(CheckoutQuoteResponse {
        business_name: merchant.1,
        currency: "RWF".to_owned(),
        items: lines,
        total: format_money(total_cents),
    })
}

async fn create_invoice_draft(
    State(state): State<AppState>,
    Path(reference): Path<String>,
    headers: HeaderMap,
    Json(request): Json<CreateInvoiceDraftRequest>,
) -> Result<(StatusCode, Json<InvoiceDraftResponse>), ApiError> {
    let customer_id = require_customer(&state.pool, &headers).await?;
    let key = request.idempotency_key.trim();
    if !(16..=128).contains(&key.len())
        || !key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(ApiError::bad_request("Invalid idempotency key"));
    }
    let expected_cents = parse_decimal_units(request.expected_total.trim(), 18, 2)
        .ok_or_else(|| ApiError::bad_request("Invalid expected total"))?;
    let expected_total = format_money(expected_cents);
    let request_bytes = serde_json::to_vec(&(&reference, &request.expected_total, &request.items))
        .map_err(|_| ApiError::internal())?;
    let request_hash = format!("{:x}", Sha256::digest(&request_bytes));

    let mut transaction = state.pool.begin().await.map_err(|_| ApiError::internal())?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(format!("invoice:{customer_id}:{key}"))
        .execute(&mut *transaction)
        .await
        .map_err(|_| ApiError::internal())?;
    let existing = sqlx::query_as::<_, (String, String, String, String, String, String)>(
        "SELECT id::text, invoice_number, total_amount::text, btrim(currency),
                payment_status, checkout_request_hash
         FROM invoices
         WHERE customer_id = $1::uuid AND checkout_idempotency_key = $2",
    )
    .bind(&customer_id)
    .bind(key)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| ApiError::internal())?;
    if let Some((id, invoice_number, total, currency, payment_status, stored_hash)) = existing {
        if stored_hash != request_hash {
            return Err(ApiError::conflict(
                "Idempotency key was used for another checkout",
            ));
        }
        transaction
            .commit()
            .await
            .map_err(|_| ApiError::internal())?;
        return Ok((
            StatusCode::OK,
            Json(InvoiceDraftResponse {
                id,
                invoice_number,
                total,
                currency,
                payment_status,
            }),
        ));
    }

    let merchant = approved_merchant_from_reference(&state.pool, &reference).await?;
    let quote = build_quote(
        &mut *transaction,
        merchant.clone(),
        CheckoutQuoteRequest {
            items: request.items,
        },
    )
    .await?;
    if quote.total != expected_total {
        return Err(ApiError::conflict("Total changed; request a new quote"));
    }

    let (id, invoice_number, total, currency, payment_status) =
        sqlx::query_as::<_, (String, String, String, String, String)>(
            "INSERT INTO invoices (
                merchant_id, customer_id, invoice_number, subtotal, total_amount,
                checkout_idempotency_key, checkout_request_hash
             ) VALUES (
                $1::uuid, $2::uuid, concat('IHUZA-', replace(gen_random_uuid()::text, '-', '')),
                $3::numeric, $3::numeric, $4, $5
             ) RETURNING id::text, invoice_number, total_amount::text,
                         btrim(currency), payment_status",
        )
        .bind(&merchant.0)
        .bind(&customer_id)
        .bind(&quote.total)
        .bind(key)
        .bind(&request_hash)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| ApiError::internal())?;

    for line in quote.items {
        sqlx::query(
            "INSERT INTO invoice_items (
                invoice_id, product_id, product_name, quantity, unit_price, subtotal
             ) VALUES ($1::uuid, $2::uuid, $3, $4::numeric, $5::numeric, $6::numeric)",
        )
        .bind(&id)
        .bind(&line.product_id)
        .bind(&line.product_name)
        .bind(&line.quantity)
        .bind(&line.unit_price)
        .bind(&line.subtotal)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ApiError::internal())?;
    }
    transaction
        .commit()
        .await
        .map_err(|_| ApiError::internal())?;
    Ok((
        StatusCode::CREATED,
        Json(InvoiceDraftResponse {
            id,
            invoice_number,
            total,
            currency,
            payment_status,
        }),
    ))
}

async fn authenticated_user_id(pool: &PgPool, headers: &HeaderMap) -> Result<String, ApiError> {
    let Some(session_token) = bearer_token(headers) else {
        return Err(ApiError::unauthorized());
    };
    sqlx::query_scalar::<_, String>(
        "SELECT u.id::text
         FROM user_sessions s JOIN users u ON u.id = s.user_id
         WHERE s.token_hash = $1 AND s.revoked_at IS NULL
           AND s.expires_at > NOW() AND u.is_active = TRUE",
    )
    .bind(hash_session_token(session_token))
    .fetch_optional(pool)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(ApiError::unauthorized)
}

type InvoiceSummaryRow = (String, String, String, String, String, String, String);

fn invoice_summary_from_row(
    (id, invoice_number, business_name, total, currency, payment_status, invoice_date): InvoiceSummaryRow,
) -> InvoiceSummaryResponse {
    InvoiceSummaryResponse {
        id,
        invoice_number,
        business_name,
        total,
        currency,
        payment_status,
        invoice_date,
    }
}

async fn list_invoices(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ProductListQuery>,
) -> Result<Json<Vec<InvoiceSummaryResponse>>, ApiError> {
    let user_id = authenticated_user_id(&state.pool, &headers).await?;
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    if !(1..=50).contains(&limit) || !(0..=100_000).contains(&offset) {
        return Err(ApiError::bad_request("Invalid invoice page"));
    }
    let rows = sqlx::query_as::<_, InvoiceSummaryRow>(
        "SELECT i.id::text, i.invoice_number, m.business_name,
                i.total_amount::text, btrim(i.currency), i.payment_status,
                i.invoice_date::text
         FROM invoices i JOIN merchant_profiles m ON m.id = i.merchant_id
         WHERE i.customer_id = $1::uuid
            OR (m.user_id = $1::uuid AND EXISTS (
                SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id
                WHERE ur.user_id = m.user_id AND r.name = 'MERCHANT'
            ))
         ORDER BY i.invoice_date DESC, i.id DESC
         LIMIT $2 OFFSET $3",
    )
    .bind(&user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    Ok(Json(
        rows.into_iter().map(invoice_summary_from_row).collect(),
    ))
}

async fn get_invoice(
    State(state): State<AppState>,
    Path(invoice_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<InvoiceDetailResponse>, ApiError> {
    let user_id = authenticated_user_id(&state.pool, &headers).await?;
    let row = sqlx::query_as::<_, InvoiceSummaryRow>(
        "SELECT i.id::text, i.invoice_number, m.business_name,
                i.total_amount::text, btrim(i.currency), i.payment_status,
                i.invoice_date::text
         FROM invoices i JOIN merchant_profiles m ON m.id = i.merchant_id
         WHERE i.id::text = $1 AND (
             i.customer_id = $2::uuid
             OR (m.user_id = $2::uuid AND EXISTS (
                 SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id
                 WHERE ur.user_id = m.user_id AND r.name = 'MERCHANT'
             ))
         )",
    )
    .bind(&invoice_id)
    .bind(&user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(|| ApiError::not_found("Invoice was not found"))?;
    let invoice = invoice_summary_from_row(row);
    let rows = sqlx::query_as::<_, (Option<String>, String, String, String, String)>(
        "SELECT product_id::text, product_name, quantity::text,
                unit_price::text, subtotal::text
         FROM invoice_items WHERE invoice_id = $1::uuid
         ORDER BY created_at, id",
    )
    .bind(&invoice.id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    let items = rows
        .into_iter()
        .map(
            |(product_id, product_name, quantity, unit_price, subtotal)| InvoiceItemResponse {
                product_id,
                product_name,
                quantity,
                unit_price,
                subtotal,
            },
        )
        .collect();
    Ok(Json(InvoiceDetailResponse { invoice, items }))
}

fn parse_decimal_units(value: &str, max_integer_digits: usize, scale: usize) -> Option<u128> {
    let (integer, fraction) = value.split_once('.').unwrap_or((value, ""));
    if integer.is_empty()
        || integer.len() > max_integer_digits
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
        || fraction.len() > scale
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
        || value.ends_with('.')
    {
        return None;
    }
    let factor = 10_u128.pow(scale as u32);
    let whole = integer.parse::<u128>().ok()?.checked_mul(factor)?;
    let fractional = if fraction.is_empty() {
        0
    } else {
        fraction.parse::<u128>().ok()? * 10_u128.pow((scale - fraction.len()) as u32)
    };
    whole.checked_add(fractional)
}

fn format_money(cents: u128) -> String {
    format!("{}.{:02}", cents / 100, cents % 100)
}

fn line_subtotal_cents(price_cents: u128, quantity_milli: u128) -> Option<u128> {
    price_cents
        .checked_mul(quantity_milli)?
        .checked_add(500)
        .map(|amount| amount / 1000)
}

fn valid_catalog_price(price: &str) -> bool {
    let mut parts = price.split('.');
    let integer = parts.next().unwrap_or_default();
    let fractional = parts.next();
    !integer.is_empty()
        && integer.len() <= 18
        && integer.bytes().all(|byte| byte.is_ascii_digit())
        && fractional.is_none_or(|digits| {
            !digits.is_empty()
                && digits.len() <= 2
                && digits.bytes().all(|byte| byte.is_ascii_digit())
        })
        && parts.next().is_none()
}

async fn account_qr_reference(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AccountQrReferenceResponse>, ApiError> {
    let Some(session_token) = bearer_token(&headers) else {
        return Err(ApiError::unauthorized());
    };

    let reference = sqlx::query_scalar::<_, String>(
        "SELECT u.account_qr_reference
         FROM user_sessions s
         JOIN users u ON u.id = s.user_id
         WHERE s.token_hash = $1
           AND s.revoked_at IS NULL
           AND s.expires_at > NOW()
           AND u.is_active = TRUE",
    )
    .bind(hash_session_token(session_token))
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(ApiError::unauthorized)?;

    Ok(Json(AccountQrReferenceResponse { reference }))
}

async fn approve_merchant_application(
    State(state): State<AppState>,
    Path(merchant_id): Path<String>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let Some(session_token) = bearer_token(&headers) else {
        return Err(ApiError::unauthorized());
    };

    let mut transaction = state.pool.begin().await.map_err(|_| ApiError::internal())?;
    let admin = sqlx::query_as::<_, (String, bool)>(
        "SELECT u.id::text, EXISTS (
             SELECT 1 FROM user_roles ur
             JOIN roles r ON r.id = ur.role_id
             WHERE ur.user_id = u.id AND r.name = 'ADMIN'
         )
         FROM user_sessions s
         JOIN users u ON u.id = s.user_id
         WHERE s.token_hash = $1
           AND s.revoked_at IS NULL
           AND s.expires_at > NOW()
           AND u.is_active = TRUE",
    )
    .bind(hash_session_token(session_token))
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(ApiError::unauthorized)?;

    if !admin.1 {
        return Err(ApiError::forbidden());
    }

    let applicant_id = sqlx::query_scalar::<_, String>(
        "UPDATE merchant_profiles
         SET status = 'APPROVED', updated_at = NOW()
         WHERE id::text = $1 AND status = 'PENDING'
         RETURNING user_id::text",
    )
    .bind(&merchant_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| ApiError::internal())?;

    let Some(applicant_id) = applicant_id else {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM merchant_profiles WHERE id::text = $1)",
        )
        .bind(&merchant_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| ApiError::internal())?;
        return if exists {
            Err(ApiError::conflict("Merchant application is not pending"))
        } else {
            Err(ApiError::not_found("Merchant application was not found"))
        };
    };

    let role_assignment = sqlx::query(
        "INSERT INTO user_roles (user_id, role_id)
         SELECT $1::uuid, id FROM roles WHERE name = 'MERCHANT'
         ON CONFLICT DO NOTHING",
    )
    .bind(&applicant_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| ApiError::internal())?;

    if role_assignment.rows_affected() != 1 {
        return Err(ApiError::internal());
    }

    sqlx::query(
        "INSERT INTO security_events (user_id, event_type, metadata)
         VALUES ($1::uuid, 'MERCHANT_APPROVED', $2)",
    )
    .bind(&admin.0)
    .bind(serde_json::json!({
        "merchant_id": merchant_id,
        "applicant_id": applicant_id,
    }))
    .execute(&mut *transaction)
    .await
    .map_err(|_| ApiError::internal())?;

    transaction
        .commit()
        .await
        .map_err(|_| ApiError::internal())?;
    Ok(StatusCode::NO_CONTENT)
}

async fn apply_for_merchant(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<MerchantApplicationRequest>,
) -> Result<(StatusCode, Json<MerchantApplicationResponse>), ApiError> {
    let Some(session_token) = bearer_token(&headers) else {
        return Err(ApiError::unauthorized());
    };

    let business_name = request.business_name.trim();
    let business_type = optional_trimmed(request.business_type);
    let phone_number = optional_trimmed(request.phone_number);
    let address = optional_trimmed(request.address);

    if business_name.is_empty() || business_name.chars().count() > 150 {
        return Err(ApiError::bad_request("A valid business name is required"));
    }
    if business_type
        .as_ref()
        .is_some_and(|value| value.chars().count() > 100)
        || phone_number
            .as_ref()
            .is_some_and(|value| value.len() > MAX_PHONE_LENGTH)
        || address
            .as_ref()
            .is_some_and(|value| value.chars().count() > 500)
    {
        return Err(ApiError::bad_request("Merchant details are too long"));
    }

    let user_id = sqlx::query_scalar::<_, String>(
        "SELECT u.id::text
         FROM user_sessions s
         JOIN users u ON u.id = s.user_id
         WHERE s.token_hash = $1
           AND s.revoked_at IS NULL
           AND s.expires_at > NOW()
           AND u.is_active = TRUE",
    )
    .bind(hash_session_token(session_token))
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(ApiError::unauthorized)?;

    let merchant_id = sqlx::query_scalar::<_, String>(
        "INSERT INTO merchant_profiles
             (user_id, business_name, business_type, phone_number, address, status)
         VALUES ($1::uuid, $2, $3, $4, $5, 'PENDING')
         RETURNING id::text",
    )
    .bind(&user_id)
    .bind(business_name)
    .bind(&business_type)
    .bind(&phone_number)
    .bind(&address)
    .fetch_one(&state.pool)
    .await;

    match merchant_id {
        Ok(id) => Ok((
            StatusCode::CREATED,
            Json(MerchantApplicationResponse {
                id,
                status: "PENDING",
            }),
        )),
        Err(error) if is_unique_violation(&error) => {
            Err(ApiError::conflict("A merchant application already exists"))
        }
        Err(_) => Err(ApiError::internal()),
    }
}

async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<StatusCode, ApiError> {
    let registration = validate_registration(request)?;
    let password_hash = hash_password(&registration.password)?;
    let mut transaction = state.pool.begin().await.map_err(|_| ApiError::internal())?;

    let user_id = match sqlx::query_scalar::<_, String>(
        "INSERT INTO users (phone_number, email, password_hash, first_name, last_name)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id::text",
    )
    .bind(&registration.phone_number)
    .bind(&registration.email)
    .bind(&password_hash)
    .bind(&registration.first_name)
    .bind(&registration.last_name)
    .fetch_one(&mut *transaction)
    .await
    {
        Ok(user_id) => user_id,
        Err(error) if is_unique_violation(&error) => {
            return Err(ApiError::conflict(
                "Phone number or email is already registered",
            ));
        }
        Err(_) => return Err(ApiError::internal()),
    };

    let role_assignment = sqlx::query(
        "INSERT INTO user_roles (user_id, role_id)
         SELECT $1::uuid, id FROM roles WHERE name = 'CUSTOMER'",
    )
    .bind(&user_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| ApiError::internal())?;

    if role_assignment.rows_affected() != 1 {
        return Err(ApiError::internal());
    }

    transaction
        .commit()
        .await
        .map_err(|_| ApiError::internal())?;

    Ok(StatusCode::CREATED)
}

async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let identifier = request.identifier.trim();

    if identifier.is_empty() || request.password.is_empty() {
        return Err(ApiError::unauthorized());
    }

    let identifier_hash = hash_identifier(identifier);

    if login_is_locked(&state.pool, &identifier_hash).await? {
        let _ = hash_password(&request.password);
        return Err(ApiError::unauthorized());
    }

    let user = sqlx::query_as::<_, (String, String, bool)>(
        "SELECT id::text, password_hash, is_active
         FROM users
         WHERE phone_number = $1 OR lower(email) = lower($1)
         LIMIT 1",
    )
    .bind(identifier)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;

    let password_is_valid = match user.as_ref() {
        Some((_, password_hash, _)) => {
            let parsed_hash = PasswordHash::new(password_hash).map_err(|_| ApiError::internal())?;
            Argon2::default()
                .verify_password(request.password.as_bytes(), &parsed_hash)
                .is_ok()
        }
        None => {
            let _ = hash_password(&request.password);
            false
        }
    };

    let Some((user_id, _, is_active)) = user else {
        record_failed_login(&state.pool, &identifier_hash, None).await?;
        return Err(ApiError::unauthorized());
    };

    if !is_active || !password_is_valid {
        record_failed_login(&state.pool, &identifier_hash, Some(&user_id)).await?;
        return Err(ApiError::unauthorized());
    }

    let session_token = generate_session_token();
    let token_hash = hash_session_token(&session_token);
    let mut transaction = state.pool.begin().await.map_err(|_| ApiError::internal())?;

    sqlx::query(
        "INSERT INTO user_sessions (user_id, token_hash, expires_at)
         VALUES ($1::uuid, $2, NOW() + ($3 * INTERVAL '1 second'))",
    )
    .bind(&user_id)
    .bind(token_hash)
    .bind(SESSION_DURATION_SECONDS)
    .execute(&mut *transaction)
    .await
    .map_err(|_| ApiError::internal())?;

    sqlx::query(
        "INSERT INTO security_events (user_id, event_type, identifier_hash)
         VALUES ($1::uuid, 'LOGIN_SUCCEEDED', $2)",
    )
    .bind(&user_id)
    .bind(&identifier_hash)
    .execute(&mut *transaction)
    .await
    .map_err(|_| ApiError::internal())?;

    sqlx::query("DELETE FROM login_rate_limits WHERE identifier_hash = $1")
        .bind(&identifier_hash)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ApiError::internal())?;

    transaction
        .commit()
        .await
        .map_err(|_| ApiError::internal())?;

    Ok(Json(LoginResponse {
        access_token: session_token,
        token_type: "Bearer".to_owned(),
        expires_in_seconds: SESSION_DURATION_SECONDS,
    }))
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Result<StatusCode, ApiError> {
    let Some(session_token) = bearer_token(&headers) else {
        return Err(ApiError::unauthorized());
    };

    let user_id = sqlx::query_scalar::<_, String>(
        "UPDATE user_sessions
         SET revoked_at = NOW()
         WHERE token_hash = $1
           AND revoked_at IS NULL
           AND expires_at > NOW()
         RETURNING user_id::text",
    )
    .bind(hash_session_token(session_token))
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;

    if let Some(user_id) = user_id {
        record_security_event(&state.pool, Some(&user_id), "LOGOUT_SUCCEEDED", None).await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn current_user(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<CurrentUserResponse>, ApiError> {
    let Some(session_token) = bearer_token(&headers) else {
        return Err(ApiError::unauthorized());
    };

    let session = sqlx::query_as::<
        _,
        (
            String,
            String,
            String,
            Option<String>,
            String,
            Option<String>,
        ),
    >(
        "SELECT s.id::text, u.id::text, u.phone_number, u.email, u.first_name, u.last_name
         FROM user_sessions s
         JOIN users u ON u.id = s.user_id
         WHERE s.token_hash = $1
           AND s.revoked_at IS NULL
           AND s.expires_at > NOW()
           AND u.is_active = TRUE",
    )
    .bind(hash_session_token(session_token))
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;

    let Some((session_id, user_id, phone_number, email, first_name, last_name)) = session else {
        return Err(ApiError::unauthorized());
    };

    let updated_session = sqlx::query(
        "UPDATE user_sessions
         SET last_used_at = NOW()
         WHERE id = $1::uuid
           AND revoked_at IS NULL
           AND expires_at > NOW()",
    )
    .bind(&session_id)
    .execute(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;

    if updated_session.rows_affected() != 1 {
        return Err(ApiError::unauthorized());
    }

    let roles = sqlx::query_scalar::<_, String>(
        "SELECT r.name
         FROM user_roles ur
         JOIN roles r ON r.id = ur.role_id
         WHERE ur.user_id = $1::uuid
         ORDER BY r.name",
    )
    .bind(&user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;

    Ok(Json(CurrentUserResponse {
        id: user_id,
        phone_number,
        email,
        first_name,
        last_name,
        roles,
    }))
}

async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<PasswordChangeRequest>,
) -> Result<StatusCode, ApiError> {
    let Some(session_token) = bearer_token(&headers) else {
        return Err(ApiError::unauthorized());
    };

    validate_password(&request.new_password)?;

    if request.current_password == request.new_password {
        return Err(ApiError::bad_request(
            "New password must be different from the current password",
        ));
    }

    let user = sqlx::query_as::<_, (String, String)>(
        "SELECT u.id::text, u.password_hash
         FROM user_sessions s
         JOIN users u ON u.id = s.user_id
         WHERE s.token_hash = $1
           AND s.revoked_at IS NULL
           AND s.expires_at > NOW()
           AND u.is_active = TRUE",
    )
    .bind(hash_session_token(session_token))
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;

    let Some((user_id, password_hash)) = user else {
        return Err(ApiError::unauthorized());
    };
    let parsed_hash = PasswordHash::new(&password_hash).map_err(|_| ApiError::internal())?;

    if Argon2::default()
        .verify_password(request.current_password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Err(ApiError::unauthorized());
    }

    let new_password_hash = hash_password(&request.new_password)?;
    let mut transaction = state.pool.begin().await.map_err(|_| ApiError::internal())?;

    sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2::uuid")
        .bind(new_password_hash)
        .bind(&user_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ApiError::internal())?;
    sqlx::query(
        "UPDATE user_sessions
         SET revoked_at = NOW()
         WHERE user_id = $1::uuid
           AND revoked_at IS NULL
           AND expires_at > NOW()",
    )
    .bind(&user_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| ApiError::internal())?;
    sqlx::query(
        "INSERT INTO security_events (user_id, event_type)
         VALUES ($1::uuid, 'PASSWORD_CHANGED')",
    )
    .bind(&user_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| ApiError::internal())?;

    transaction
        .commit()
        .await
        .map_err(|_| ApiError::internal())?;

    Ok(StatusCode::NO_CONTENT)
}

fn validate_registration(request: RegisterRequest) -> Result<NewRegistration, ApiError> {
    let phone_number = request.phone_number.trim().to_owned();
    let first_name = request.first_name.trim().to_owned();
    let last_name = optional_trimmed(request.last_name);
    let email = optional_trimmed(request.email);

    if phone_number.is_empty() || phone_number.len() > MAX_PHONE_LENGTH {
        return Err(ApiError::bad_request("A valid phone number is required"));
    }

    if first_name.is_empty() || first_name.chars().count() > MAX_NAME_LENGTH {
        return Err(ApiError::bad_request("A valid first name is required"));
    }

    if last_name
        .as_ref()
        .is_some_and(|name| name.chars().count() > MAX_NAME_LENGTH)
    {
        return Err(ApiError::bad_request("Last name is too long"));
    }

    if email
        .as_ref()
        .is_some_and(|value| value.len() > MAX_EMAIL_LENGTH || !value.contains('@'))
    {
        return Err(ApiError::bad_request("A valid email address is required"));
    }

    validate_password(&request.password)?;

    Ok(NewRegistration {
        phone_number,
        email,
        password: request.password,
        first_name,
        last_name,
    })
}

fn validate_password(password: &str) -> Result<(), ApiError> {
    if password.chars().count() < MIN_PASSWORD_LENGTH {
        return Err(ApiError::bad_request(
            "Password must contain at least 12 characters",
        ));
    }

    Ok(())
}

fn optional_trimmed(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        let item = item.trim().to_owned();
        (!item.is_empty()).then_some(item)
    })
}

fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| ApiError::internal())
}

fn generate_session_token() -> String {
    let mut token_bytes = [0_u8; SESSION_TOKEN_BYTES];
    OsRng.fill_bytes(&mut token_bytes);
    URL_SAFE_NO_PAD.encode(token_bytes)
}

fn hash_session_token(session_token: &str) -> String {
    format!("{:x}", Sha256::digest(session_token.as_bytes()))
}

fn hash_identifier(identifier: &str) -> String {
    format!("{:x}", Sha256::digest(identifier.to_lowercase().as_bytes()))
}

async fn login_is_locked(pool: &PgPool, identifier_hash: &str) -> Result<bool, ApiError> {
    sqlx::query_scalar::<_, bool>(
        "SELECT COALESCE(locked_until > NOW(), FALSE)
         FROM login_rate_limits
         WHERE identifier_hash = $1",
    )
    .bind(identifier_hash)
    .fetch_optional(pool)
    .await
    .map(|is_locked| is_locked.unwrap_or(false))
    .map_err(|_| ApiError::internal())
}

async fn record_failed_login(
    pool: &PgPool,
    identifier_hash: &str,
    user_id: Option<&str>,
) -> Result<(), ApiError> {
    let mut transaction = pool.begin().await.map_err(|_| ApiError::internal())?;

    sqlx::query(
        "INSERT INTO security_events (user_id, event_type, identifier_hash)
         VALUES ($1::uuid, 'LOGIN_FAILED', $2)",
    )
    .bind(user_id)
    .bind(identifier_hash)
    .execute(&mut *transaction)
    .await
    .map_err(|_| ApiError::internal())?;

    sqlx::query(
        "INSERT INTO login_rate_limits (
             identifier_hash,
             failed_attempts,
             window_started_at,
             locked_until,
             updated_at
         )
         VALUES ($1, 1, NOW(), NULL, NOW())
         ON CONFLICT (identifier_hash) DO UPDATE
         SET failed_attempts = CASE
                 WHEN login_rate_limits.locked_until > NOW()
                     THEN login_rate_limits.failed_attempts
                 WHEN login_rate_limits.window_started_at <= NOW() - ($2 * INTERVAL '1 second')
                     THEN 1
                 ELSE login_rate_limits.failed_attempts + 1
             END,
             window_started_at = CASE
                 WHEN login_rate_limits.locked_until > NOW()
                     THEN login_rate_limits.window_started_at
                 WHEN login_rate_limits.window_started_at <= NOW() - ($2 * INTERVAL '1 second')
                     THEN NOW()
                 ELSE login_rate_limits.window_started_at
             END,
             locked_until = CASE
                 WHEN login_rate_limits.locked_until > NOW()
                     THEN login_rate_limits.locked_until
                 WHEN login_rate_limits.window_started_at <= NOW() - ($2 * INTERVAL '1 second')
                     THEN NULL
                 WHEN login_rate_limits.failed_attempts + 1 >= $3
                     THEN NOW() + ($4 * INTERVAL '1 second')
                 ELSE NULL
             END,
             updated_at = NOW()",
    )
    .bind(identifier_hash)
    .bind(LOGIN_FAILURE_WINDOW_SECONDS)
    .bind(MAX_FAILED_LOGIN_ATTEMPTS)
    .bind(LOGIN_LOCKOUT_SECONDS)
    .execute(&mut *transaction)
    .await
    .map_err(|_| ApiError::internal())?;

    transaction.commit().await.map_err(|_| ApiError::internal())
}

async fn record_security_event(
    pool: &PgPool,
    user_id: Option<&str>,
    event_type: &'static str,
    identifier_hash: Option<&str>,
) -> Result<(), ApiError> {
    sqlx::query(
        "INSERT INTO security_events (user_id, event_type, identifier_hash)
         VALUES ($1::uuid, $2, $3)",
    )
    .bind(user_id)
    .bind(event_type)
    .bind(identifier_hash)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(|_| ApiError::internal())
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    matches!(
        error,
        sqlx::Error::Database(database_error)
            if database_error.code().as_deref() == Some("23505")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use std::time::{SystemTime, UNIX_EPOCH};
    use tower::ServiceExt;

    fn valid_request() -> RegisterRequest {
        RegisterRequest {
            phone_number: "  +250788123456  ".to_owned(),
            email: Some("  citizen@example.com  ".to_owned()),
            password: "a secure test password".to_owned(),
            first_name: "  Aline  ".to_owned(),
            last_name: Some("  Uwimana  ".to_owned()),
        }
    }

    #[test]
    fn registration_validation_trims_optional_fields() {
        let registration = validate_registration(valid_request()).expect("request should be valid");

        assert_eq!(registration.phone_number, "+250788123456");
        assert_eq!(registration.email.as_deref(), Some("citizen@example.com"));
        assert_eq!(registration.first_name, "Aline");
        assert_eq!(registration.last_name.as_deref(), Some("Uwimana"));
    }

    #[test]
    fn registration_validation_rejects_short_passwords() {
        let mut request = valid_request();
        request.password = "short".to_owned();

        let error = validate_registration(request).expect_err("short password must be rejected");

        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(
            error.message,
            "Password must contain at least 12 characters"
        );
    }

    #[test]
    fn password_hash_verifies_without_exposing_the_password() {
        let password = "a secure test password";
        let hash = hash_password(password).expect("password hashing should succeed");
        let parsed_hash = PasswordHash::new(&hash).expect("hash must use a valid PHC format");

        assert_ne!(hash, password);
        assert!(
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok()
        );
    }

    #[test]
    fn session_tokens_are_random_and_only_their_hash_is_stored() {
        let first_token = generate_session_token();
        let second_token = generate_session_token();
        let first_hash = hash_session_token(&first_token);

        assert_ne!(first_token, second_token);
        assert_ne!(first_token, first_hash);
        assert_eq!(first_hash, hash_session_token(&first_token));
        assert_eq!(first_hash.len(), 64);
    }

    #[test]
    fn identifier_hash_is_case_insensitive_without_retaining_the_identifier() {
        let first_hash = hash_identifier("Citizen@Example.Test");
        let second_hash = hash_identifier("citizen@example.test");

        assert_eq!(first_hash, second_hash);
        assert_ne!(first_hash, "citizen@example.test");
        assert_eq!(first_hash.len(), 64);
    }

    #[test]
    fn bearer_token_requires_the_expected_scheme_and_value() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer session-token".parse().unwrap());

        assert_eq!(bearer_token(&headers), Some("session-token"));

        headers.insert("authorization", "Basic credentials".parse().unwrap());
        assert_eq!(bearer_token(&headers), None);
    }

    #[test]
    fn catalog_price_accepts_exact_decimal_values_only() {
        for price in ["0", "5000", "1500.50", "999999999999999999.99"] {
            assert!(valid_catalog_price(price), "{price} should be valid");
        }
        for price in ["", "-1", "1.234", "1.", ".5", "1e3", "1,000", "1.2.3"] {
            assert!(!valid_catalog_price(price), "{price} should be rejected");
        }
    }

    #[test]
    fn quote_decimal_math_rounds_each_line_to_two_places() {
        assert_eq!(parse_decimal_units("0.333", 17, 3), Some(333));
        assert_eq!(parse_decimal_units("1500.50", 18, 2), Some(150_050));
        assert_eq!(line_subtotal_cents(150_050, 333), Some(49_967));
        assert_eq!(format_money(49_967), "499.67");
        assert_eq!(line_subtotal_cents(1, 500), Some(1));
        assert_eq!(parse_decimal_units("1.2345", 17, 3), None);
        assert_eq!(line_subtotal_cents(u128::MAX, 2), None);
    }

    #[tokio::test]
    async fn product_creation_requires_a_bearer_token() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://ignored:ignored@127.0.0.1:5432/ihuza")
            .expect("a lazy test pool should be valid");
        let request = Request::builder()
            .method("POST")
            .uri("/merchant/products")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"name":"Rice","price":"1500.50"}"#))
            .expect("request should be valid");

        let response = app(pool)
            .oneshot(request)
            .await
            .expect("the route should return a response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn product_list_requires_a_bearer_token() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://ignored:ignored@127.0.0.1:5432/ihuza")
            .expect("a lazy test pool should be valid");
        let request = Request::builder()
            .method("GET")
            .uri("/merchant/products")
            .body(Body::empty())
            .expect("request should be valid");

        let response = app(pool)
            .oneshot(request)
            .await
            .expect("the route should return a response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn product_price_update_requires_a_bearer_token() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://ignored:ignored@127.0.0.1:5432/ihuza")
            .expect("a lazy test pool should be valid");
        let request = Request::builder()
            .method("PATCH")
            .uri("/merchant/products/00000000-0000-0000-0000-000000000000")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"price":"1600.00"}"#))
            .expect("request should be valid");

        let response = app(pool)
            .oneshot(request)
            .await
            .expect("the route should return a response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn merchant_catalog_requires_a_bearer_token() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://ignored:ignored@127.0.0.1:5432/ihuza")
            .expect("a lazy test pool should be valid");
        let request = Request::builder()
            .method("GET")
            .uri("/qr/000000000000000000000000000000000000000000000000/catalog")
            .body(Body::empty())
            .expect("request should be valid");

        let response = app(pool)
            .oneshot(request)
            .await
            .expect("the route should return a response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn merchant_quote_requires_a_bearer_token() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://ignored:ignored@127.0.0.1:5432/ihuza")
            .expect("a lazy test pool should be valid");
        let request = Request::builder()
            .method("POST")
            .uri("/qr/000000000000000000000000000000000000000000000000/quote")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"items":[{"product_id":"00000000-0000-0000-0000-000000000000","quantity":"1"}]}"#))
            .expect("request should be valid");

        let response = app(pool)
            .oneshot(request)
            .await
            .expect("the route should return a response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn invoice_draft_requires_a_bearer_token() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://ignored:ignored@127.0.0.1:5432/ihuza")
            .expect("a lazy test pool should be valid");
        let request = Request::builder()
            .method("POST")
            .uri("/qr/000000000000000000000000000000000000000000000000/invoices")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"items":[],"expected_total":"0.00","idempotency_key":"test-invoice-key-123"}"#,
            ))
            .expect("request should be valid");
        let response = app(pool)
            .oneshot(request)
            .await
            .expect("the route should return a response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn invoice_reads_require_a_bearer_token() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://ignored:ignored@127.0.0.1:5432/ihuza")
            .expect("a lazy test pool should be valid");
        for uri in [
            "/invoices",
            "/invoices/00000000-0000-0000-0000-000000000000",
        ] {
            let request = Request::builder()
                .method("GET")
                .uri(uri)
                .body(Body::empty())
                .expect("request should be valid");
            let response = app(pool.clone())
                .oneshot(request)
                .await
                .expect("the route should return a response");
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
    }

    #[tokio::test]
    async fn register_route_rejects_short_password_before_database_write() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://ignored:ignored@127.0.0.1:5432/ihuza")
            .expect("a lazy test pool should be valid");
        let request = Request::builder()
            .method("POST")
            .uri("/auth/register")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"phone_number":"+250788000000","password":"short","first_name":"Test"}"#,
            ))
            .expect("request should be valid");

        let response = app(pool)
            .oneshot(request)
            .await
            .expect("the route should return a response");
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(
            std::str::from_utf8(&body).expect("response should be UTF-8"),
            "Password must contain at least 12 characters"
        );
    }

    #[tokio::test]
    async fn logout_route_requires_a_bearer_token_before_database_access() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://ignored:ignored@127.0.0.1:5432/ihuza")
            .expect("a lazy test pool should be valid");
        let request = Request::builder()
            .method("POST")
            .uri("/auth/logout")
            .body(Body::empty())
            .expect("request should be valid");

        let response = app(pool)
            .oneshot(request)
            .await
            .expect("the route should return a response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn current_user_route_requires_a_bearer_token_before_database_access() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://ignored:ignored@127.0.0.1:5432/ihuza")
            .expect("a lazy test pool should be valid");
        let request = Request::builder()
            .method("GET")
            .uri("/auth/me")
            .body(Body::empty())
            .expect("request should be valid");

        let response = app(pool)
            .oneshot(request)
            .await
            .expect("the route should return a response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn account_qr_reference_requires_a_bearer_token() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://ignored:ignored@127.0.0.1:5432/ihuza")
            .expect("a lazy test pool should be valid");
        let request = Request::builder()
            .method("GET")
            .uri("/account/qr-reference")
            .body(Body::empty())
            .expect("request should be valid");

        let response = app(pool)
            .oneshot(request)
            .await
            .expect("the route should return a response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn wallet_routes_require_a_bearer_token() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://ignored:ignored@127.0.0.1:5432/ihuza")
            .expect("a lazy test pool should be valid");
        for method in ["GET", "POST"] {
            let request = Request::builder()
                .method(method)
                .uri("/wallet")
                .body(Body::empty())
                .expect("request should be valid");
            let response = app(pool.clone())
                .oneshot(request)
                .await
                .expect("the route should return a response");
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL; wallet fixture rows are rolled back"]
    async fn wallet_provisioning_is_zero_balance_and_idempotent() {
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
        let user_id: String = sqlx::query_scalar(
            "INSERT INTO users (phone_number, password_hash, first_name)
             VALUES ($1, 'wallet-provision-test-only-hash', 'Wallet Test') RETURNING id::text",
        )
        .bind(format!("wallet-provision-{suffix}"))
        .fetch_one(&mut *tx)
        .await
        .expect("test user should be created");

        let before_role = provision_wallet(&mut tx, &user_id).await;
        assert!(before_role.is_err(), "a non-customer cannot open a wallet");
        let count_before_role: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM wallet_accounts WHERE user_id = $1::uuid")
                .bind(&user_id)
                .fetch_one(&mut *tx)
                .await
                .expect("wallet count should be readable");
        assert_eq!(count_before_role, 0);

        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id)
             SELECT $1::uuid, id FROM roles WHERE name = 'CUSTOMER'",
        )
        .bind(&user_id)
        .execute(&mut *tx)
        .await
        .expect("customer role should be assigned");

        for _ in 0..2 {
            let summary = provision_wallet(&mut tx, &user_id)
                .await
                .expect("customer wallet should open or be returned");
            assert_eq!(summary.currency, "RWF");
            assert_eq!(summary.balance, "0.00");
            assert_eq!(summary.status, "ACTIVE");
            assert!(!summary.money_operations_enabled);
        }
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM wallet_accounts WHERE user_id = $1::uuid")
                .bind(&user_id)
                .fetch_one(&mut *tx)
                .await
                .expect("wallet count should be readable");
        assert_eq!(count, 1);

        sqlx::query("UPDATE wallet_accounts SET status = 'FROZEN' WHERE user_id = $1::uuid")
            .bind(&user_id)
            .execute(&mut *tx)
            .await
            .expect("test wallet should be frozen");
        let frozen = provision_wallet(&mut tx, &user_id)
            .await
            .expect("existing wallet should be returned without changing its status");
        assert_eq!(frozen.status, "FROZEN");
        assert_eq!(frozen.balance, "0.00");

        tx.rollback()
            .await
            .expect("wallet fixture should roll back");
        let remaining: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM wallet_accounts WHERE user_id = $1::uuid")
                .bind(&user_id)
                .fetch_one(&pool)
                .await
                .expect("rollback should be verifiable");
        assert_eq!(remaining, 0);
    }

    #[tokio::test]
    async fn account_qr_resolution_requires_a_bearer_token() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://ignored:ignored@127.0.0.1:5432/ihuza")
            .expect("a lazy test pool should be valid");
        let request = Request::builder()
            .method("GET")
            .uri("/qr/000000000000000000000000000000000000000000000000")
            .body(Body::empty())
            .expect("request should be valid");
        let response = app(pool)
            .oneshot(request)
            .await
            .expect("the route should return a response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn password_change_route_requires_a_bearer_token_before_database_access() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://ignored:ignored@127.0.0.1:5432/ihuza")
            .expect("a lazy test pool should be valid");
        let request = Request::builder()
            .method("PATCH")
            .uri("/auth/password")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"current_password":"old password","new_password":"a secure replacement password"}"#,
            ))
            .expect("request should be valid");

        let response = app(pool)
            .oneshot(request)
            .await
            .expect("the route should return a response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn merchant_application_requires_a_bearer_token() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://ignored:ignored@127.0.0.1:5432/ihuza")
            .expect("a lazy test pool should be valid");
        let request = Request::builder()
            .method("POST")
            .uri("/merchants/apply")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"business_name":"Test Shop"}"#))
            .expect("request should be valid");

        let response = app(pool)
            .oneshot(request)
            .await
            .expect("the route should return a response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn merchant_approval_requires_a_bearer_token() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://ignored:ignored@127.0.0.1:5432/ihuza")
            .expect("a lazy test pool should be valid");
        let request = Request::builder()
            .method("POST")
            .uri("/admin/merchant-applications/00000000-0000-0000-0000-000000000001/approve")
            .body(Body::empty())
            .expect("request should be valid");

        let response = app(pool)
            .oneshot(request)
            .await
            .expect("the route should return a response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL and writes then removes isolated test data"]
    async fn only_admin_can_approve_a_pending_merchant() {
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
        let password_hash = hash_password("a secure approval test password")
            .expect("test password hashing should succeed");
        let admin_id = sqlx::query_scalar::<_, String>(
            "INSERT INTO users (phone_number, password_hash, first_name)
             VALUES ($1, $2, 'Admin Test') RETURNING id::text",
        )
        .bind(format!("admin-approval-{suffix}"))
        .bind(&password_hash)
        .fetch_one(&pool)
        .await
        .expect("admin fixture should be created");
        let applicant_id = sqlx::query_scalar::<_, String>(
            "INSERT INTO users (phone_number, password_hash, first_name)
             VALUES ($1, $2, 'Applicant Test') RETURNING id::text",
        )
        .bind(format!("applicant-approval-{suffix}"))
        .bind(&password_hash)
        .fetch_one(&pool)
        .await
        .expect("applicant fixture should be created");
        let outsider_id = sqlx::query_scalar::<_, String>(
            "INSERT INTO users (phone_number, password_hash, first_name)
             VALUES ($1, $2, 'Outsider Test') RETURNING id::text",
        )
        .bind(format!("outsider-approval-{suffix}"))
        .bind(&password_hash)
        .fetch_one(&pool)
        .await
        .expect("outsider fixture should be created");
        let merchant_id = sqlx::query_scalar::<_, String>(
            "INSERT INTO merchant_profiles (user_id, business_name)
             VALUES ($1::uuid, 'Approval Test Shop') RETURNING id::text",
        )
        .bind(&applicant_id)
        .fetch_one(&pool)
        .await
        .expect("merchant application fixture should be created");

        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id)
             SELECT $1::uuid, id FROM roles WHERE name = 'ADMIN'",
        )
        .bind(&admin_id)
        .execute(&pool)
        .await
        .expect("admin role fixture should be created");
        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id)
             SELECT $1::uuid, id FROM roles WHERE name = 'CUSTOMER'",
        )
        .bind(&admin_id)
        .execute(&pool)
        .await
        .expect("customer role fixture should be created");
        for user_id in [&applicant_id, &outsider_id] {
            sqlx::query(
                "INSERT INTO user_roles (user_id, role_id)
                 SELECT $1::uuid, id FROM roles WHERE name = 'CUSTOMER'",
            )
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("customer role fixture should be created");
        }
        let admin_token = generate_session_token();
        let applicant_token = generate_session_token();
        let outsider_token = generate_session_token();
        for (user_id, token) in [
            (&admin_id, &admin_token),
            (&applicant_id, &applicant_token),
            (&outsider_id, &outsider_token),
        ] {
            sqlx::query(
                "INSERT INTO user_sessions (user_id, token_hash, expires_at)
                 VALUES ($1::uuid, $2, NOW() + INTERVAL '1 hour')",
            )
            .bind(user_id)
            .bind(hash_session_token(token))
            .execute(&pool)
            .await
            .expect("session fixture should be created");
        }

        let test_result: Result<(), String> = async {
            let merchant_reference: String =
                sqlx::query_scalar("SELECT account_qr_reference FROM users WHERE id = $1::uuid")
                    .bind(&applicant_id)
                    .fetch_one(&pool)
                    .await
                    .map_err(|error| error.to_string())?;
            let pending_destination_request = Request::builder()
                .method("GET")
                .uri(format!("/qr/{merchant_reference}"))
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .map_err(|error| error.to_string())?;
            let pending_destination_response = app(pool.clone())
                .oneshot(pending_destination_request)
                .await
                .map_err(|error| error.to_string())?;
            if pending_destination_response.status() != StatusCode::OK {
                return Err("pending merchant QR was not resolved as a person".to_owned());
            }
            let pending_body = axum::body::to_bytes(
                pending_destination_response.into_body(),
                usize::MAX,
            )
            .await
            .map_err(|error| error.to_string())?;
            let pending_destination: serde_json::Value =
                serde_json::from_slice(&pending_body).map_err(|error| error.to_string())?;
            if pending_destination["account_type"] != "PERSON_TRANSFER"
                || pending_destination["display_name"] != "Applicant Test"
                || pending_destination["currency"] != "RWF"
            {
                return Err("pending merchant QR exposed the wrong destination".to_owned());
            }
            let pending_catalog_request = Request::builder()
                .method("GET")
                .uri(format!("/qr/{merchant_reference}/catalog"))
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .map_err(|error| error.to_string())?;
            let pending_catalog_response = app(pool.clone())
                .oneshot(pending_catalog_request)
                .await
                .map_err(|error| error.to_string())?;
            if pending_catalog_response.status() != StatusCode::NOT_FOUND {
                return Err("pending merchant exposed a catalog".to_owned());
            }

            let pending_product = Request::builder()
                .method("POST")
                .uri("/merchant/products")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {applicant_token}"))
                .body(Body::from(r#"{"name":"Rice","price":"1500.50"}"#))
                .map_err(|error| error.to_string())?;
            let pending_response = app(pool.clone())
                .oneshot(pending_product)
                .await
                .map_err(|error| error.to_string())?;
            if pending_response.status() != StatusCode::FORBIDDEN {
                return Err(format!(
                    "pending merchant product creation returned {}",
                    pending_response.status()
                ));
            }

            for (token, expected_status) in [
                (&applicant_token, StatusCode::FORBIDDEN),
                (&admin_token, StatusCode::NO_CONTENT),
                (&admin_token, StatusCode::CONFLICT),
            ] {
                let request = Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/admin/merchant-applications/{merchant_id}/approve"
                    ))
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .map_err(|error| error.to_string())?;
                let response = app(pool.clone())
                    .oneshot(request)
                    .await
                    .map_err(|error| error.to_string())?;
                if response.status() != expected_status {
                    return Err(format!("approval returned {}", response.status()));
                }
            }

            let (status, merchant_roles, audit_events) = sqlx::query_as::<_, (String, i64, i64)>(
                "SELECT m.status,
                    (SELECT COUNT(*) FROM user_roles ur JOIN roles r ON r.id = ur.role_id
                     WHERE ur.user_id = m.user_id AND r.name = 'MERCHANT'),
                    (SELECT COUNT(*) FROM security_events e
                     WHERE e.user_id = $2::uuid AND e.event_type = 'MERCHANT_APPROVED'
                       AND e.metadata->>'merchant_id' = $1)
                 FROM merchant_profiles m WHERE m.id::text = $1",
            )
            .bind(&merchant_id)
            .bind(&admin_id)
            .fetch_one(&pool)
            .await
            .map_err(|error| error.to_string())?;
            if status != "APPROVED" || merchant_roles != 1 || audit_events != 1 {
                return Err("approval status, role, or audit event was incorrect".to_owned());
            }
            let merchant_destination_request = Request::builder()
                .method("GET")
                .uri(format!("/qr/{merchant_reference}"))
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .map_err(|error| error.to_string())?;
            let merchant_destination_response = app(pool.clone())
                .oneshot(merchant_destination_request)
                .await
                .map_err(|error| error.to_string())?;
            if merchant_destination_response.status() != StatusCode::OK {
                return Err("approved merchant QR was not resolved".to_owned());
            }
            let merchant_body = axum::body::to_bytes(
                merchant_destination_response.into_body(),
                usize::MAX,
            )
            .await
            .map_err(|error| error.to_string())?;
            let merchant_destination: serde_json::Value =
                serde_json::from_slice(&merchant_body).map_err(|error| error.to_string())?;
            if merchant_destination["account_type"] != "MERCHANT_CHECKOUT"
                || merchant_destination["display_name"] != "Approval Test Shop"
            {
                return Err("approved merchant QR exposed the wrong destination".to_owned());
            }

            let product_request = Request::builder()
                .method("POST")
                .uri("/merchant/products")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {applicant_token}"))
                .body(Body::from(
                    r#"{"name":"Rice","price":"1500.50","sku":"RICE-1"}"#,
                ))
                .map_err(|error| error.to_string())?;
            let product_response = app(pool.clone())
                .oneshot(product_request)
                .await
                .map_err(|error| error.to_string())?;
            if product_response.status() != StatusCode::CREATED {
                return Err(format!(
                    "product creation returned {}",
                    product_response.status()
                ));
            }
            let body = axum::body::to_bytes(product_response.into_body(), usize::MAX)
                .await
                .map_err(|error| error.to_string())?;
            let product: CreatedProductResponse =
                serde_json::from_slice(&body).map_err(|error| error.to_string())?;
            if product.price != "1500.50" || product.currency != "RWF" {
                return Err("product price or currency changed unexpectedly".to_owned());
            }
            let owner_id: String =
                sqlx::query_scalar("SELECT merchant_id::text FROM products WHERE id::text = $1")
                    .bind(&product.id)
                    .fetch_one(&pool)
                    .await
                    .map_err(|error| error.to_string())?;
            if owner_id != merchant_id {
                return Err("product was not scoped to the approved merchant".to_owned());
            }

            let update_request = Request::builder()
                .method("PATCH")
                .uri(format!("/merchant/products/{}", product.id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {applicant_token}"))
                .body(Body::from(r#"{"price":"1600.00"}"#))
                .map_err(|error| error.to_string())?;
            let update_response = app(pool.clone())
                .oneshot(update_request)
                .await
                .map_err(|error| error.to_string())?;
            if update_response.status() != StatusCode::OK {
                return Err(format!(
                    "product price update returned {}",
                    update_response.status()
                ));
            }
            let update_body = axum::body::to_bytes(update_response.into_body(), usize::MAX)
                .await
                .map_err(|error| error.to_string())?;
            let updated: CreatedProductResponse =
                serde_json::from_slice(&update_body).map_err(|error| error.to_string())?;
            if updated.id != product.id || updated.price != "1600.00" {
                return Err("merchant price update did not return the new price".to_owned());
            }

            let list_request = Request::builder()
                .method("GET")
                .uri("/merchant/products?limit=1&offset=0")
                .header("authorization", format!("Bearer {applicant_token}"))
                .body(Body::empty())
                .map_err(|error| error.to_string())?;
            let list_response = app(pool.clone())
                .oneshot(list_request)
                .await
                .map_err(|error| error.to_string())?;
            if list_response.status() != StatusCode::OK {
                return Err(format!("product list returned {}", list_response.status()));
            }
            let list_body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
                .await
                .map_err(|error| error.to_string())?;
            let listed: Vec<CreatedProductResponse> =
                serde_json::from_slice(&list_body).map_err(|error| error.to_string())?;
            if listed.len() != 1 || listed[0].id != product.id || listed[0].price != "1600.00" {
                return Err("approved merchant could not list the new product".to_owned());
            }
            let catalog_request = Request::builder()
                .method("GET")
                .uri(format!("/qr/{merchant_reference}/catalog?limit=1&offset=0"))
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .map_err(|error| error.to_string())?;
            let catalog_response = app(pool.clone())
                .oneshot(catalog_request)
                .await
                .map_err(|error| error.to_string())?;
            if catalog_response.status() != StatusCode::OK {
                return Err(format!("QR catalog returned {}", catalog_response.status()));
            }
            let catalog_body = axum::body::to_bytes(catalog_response.into_body(), usize::MAX)
                .await
                .map_err(|error| error.to_string())?;
            let catalog: MerchantCatalogResponse =
                serde_json::from_slice(&catalog_body).map_err(|error| error.to_string())?;
            if catalog.business_name != "Approval Test Shop"
                || catalog.products.len() != 1
                || catalog.products[0].id != product.id
                || catalog.products[0].price != "1600.00"
            {
                return Err("QR catalog did not show the approved merchant product".to_owned());
            }

            let quote_request = Request::builder()
                .method("POST")
                .uri(format!("/qr/{merchant_reference}/quote"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::from(format!(
                    r#"{{"items":[{{"product_id":"{}","quantity":"0.333"}}]}}"#,
                    product.id
                )))
                .map_err(|error| error.to_string())?;
            let quote_response = app(pool.clone())
                .oneshot(quote_request)
                .await
                .map_err(|error| error.to_string())?;
            if quote_response.status() != StatusCode::OK {
                return Err(format!("QR quote returned {}", quote_response.status()));
            }
            let quote_body = axum::body::to_bytes(quote_response.into_body(), usize::MAX)
                .await
                .map_err(|error| error.to_string())?;
            let quote: CheckoutQuoteResponse =
                serde_json::from_slice(&quote_body).map_err(|error| error.to_string())?;
            if quote.business_name != "Approval Test Shop"
                || quote.currency != "RWF"
                || quote.total != "532.80"
                || quote.items.len() != 1
                || quote.items[0].product_id != product.id
                || quote.items[0].quantity != "0.333"
                || quote.items[0].unit_price != "1600.00"
                || quote.items[0].subtotal != "532.80"
            {
                return Err("QR quote did not use the merchant's current price".to_owned());
            }

            let unavailable_request = Request::builder()
                .method("POST")
                .uri(format!("/qr/{merchant_reference}/quote"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::from(r#"{"items":[{"product_id":"00000000-0000-0000-0000-000000000000","quantity":"1"}]}"#))
                .map_err(|error| error.to_string())?;
            let unavailable_response = app(pool.clone())
                .oneshot(unavailable_request)
                .await
                .map_err(|error| error.to_string())?;
            if unavailable_response.status() != StatusCode::BAD_REQUEST {
                return Err("QR quote accepted an unavailable product".to_owned());
            }

            let draft_key = format!("draft-approval-{suffix}");
            let mut first_draft_id = String::new();
            for (expected_status, total) in [
                (StatusCode::CREATED, "532.80"),
                (StatusCode::OK, "532.80"),
                (StatusCode::CONFLICT, "1.00"),
            ] {
                let draft_request = Request::builder()
                    .method("POST")
                    .uri(format!("/qr/{merchant_reference}/invoices"))
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {admin_token}"))
                    .body(Body::from(format!(
                        r#"{{"items":[{{"product_id":"{}","quantity":"0.333"}}],"expected_total":"{total}","idempotency_key":"{draft_key}"}}"#,
                        product.id
                    )))
                    .map_err(|error| error.to_string())?;
                let draft_response = app(pool.clone())
                    .oneshot(draft_request)
                    .await
                    .map_err(|error| error.to_string())?;
                if draft_response.status() != expected_status {
                    return Err(format!("invoice draft returned {}", draft_response.status()));
                }
                if expected_status != StatusCode::CONFLICT {
                    let body = axum::body::to_bytes(draft_response.into_body(), usize::MAX)
                        .await
                        .map_err(|error| error.to_string())?;
                    let draft: InvoiceDraftResponse =
                        serde_json::from_slice(&body).map_err(|error| error.to_string())?;
                    if draft.total != "532.80"
                        || draft.currency != "RWF"
                        || draft.payment_status != "PENDING"
                    {
                        return Err("invoice draft fields were incorrect".to_owned());
                    }
                    if first_draft_id.is_empty() {
                        first_draft_id = draft.id;
                    } else if first_draft_id != draft.id {
                        return Err("idempotent retry created a second invoice".to_owned());
                    }
                }
            }
            let new_price_request = Request::builder()
                .method("PATCH")
                .uri(format!("/merchant/products/{}", product.id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {applicant_token}"))
                .body(Body::from(r#"{"price":"1700.00"}"#))
                .map_err(|error| error.to_string())?;
            let new_price_response = app(pool.clone())
                .oneshot(new_price_request)
                .await
                .map_err(|error| error.to_string())?;
            if new_price_response.status() != StatusCode::OK {
                return Err("merchant could not change price after invoice draft".to_owned());
            }
            let retry_request = Request::builder()
                .method("POST")
                .uri(format!("/qr/{merchant_reference}/invoices"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::from(format!(
                    r#"{{"items":[{{"product_id":"{}","quantity":"0.333"}}],"expected_total":"532.80","idempotency_key":"{draft_key}"}}"#,
                    product.id
                )))
                .map_err(|error| error.to_string())?;
            let retry_response = app(pool.clone())
                .oneshot(retry_request)
                .await
                .map_err(|error| error.to_string())?;
            if retry_response.status() != StatusCode::OK {
                return Err("idempotent retry failed after a catalog price change".to_owned());
            }
            let retry_body = axum::body::to_bytes(retry_response.into_body(), usize::MAX)
                .await
                .map_err(|error| error.to_string())?;
            let retried: InvoiceDraftResponse =
                serde_json::from_slice(&retry_body).map_err(|error| error.to_string())?;
            if retried.id != first_draft_id || retried.total != "532.80" {
                return Err("retry changed the original invoice draft".to_owned());
            }
            let snapshot = sqlx::query_as::<_, (String, String, String)>(
                "SELECT product_name, unit_price::text, subtotal::text
                 FROM invoice_items WHERE invoice_id = $1::uuid",
            )
            .bind(&first_draft_id)
            .fetch_one(&pool)
            .await
            .map_err(|error| error.to_string())?;
            if snapshot != ("Rice".to_owned(), "1600.00".to_owned(), "532.80".to_owned()) {
                return Err("invoice item snapshot was incorrect".to_owned());
            }

            for (token, can_read) in [
                (&admin_token, true),
                (&applicant_token, true),
                (&outsider_token, false),
            ] {
                let detail_request = Request::builder()
                    .method("GET")
                    .uri(format!("/invoices/{first_draft_id}"))
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .map_err(|error| error.to_string())?;
                let detail_response = app(pool.clone())
                    .oneshot(detail_request)
                    .await
                    .map_err(|error| error.to_string())?;
                let expected_status = if can_read { StatusCode::OK } else { StatusCode::NOT_FOUND };
                if detail_response.status() != expected_status {
                    return Err(format!("invoice detail access returned {}", detail_response.status()));
                }
                if can_read {
                    let body = axum::body::to_bytes(detail_response.into_body(), usize::MAX)
                        .await
                        .map_err(|error| error.to_string())?;
                    let detail: InvoiceDetailResponse =
                        serde_json::from_slice(&body).map_err(|error| error.to_string())?;
                    if detail.invoice.id != first_draft_id
                        || detail.invoice.total != "532.80"
                        || detail.invoice.payment_status != "PENDING"
                        || detail.items.len() != 1
                        || detail.items[0].product_name != "Rice"
                        || detail.items[0].quantity != "0.333"
                        || detail.items[0].unit_price != "1600.00"
                    {
                        return Err("invoice detail lost its item snapshot".to_owned());
                    }
                }

                let list_request = Request::builder()
                    .method("GET")
                    .uri("/invoices?limit=10&offset=0")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .map_err(|error| error.to_string())?;
                let list_response = app(pool.clone())
                    .oneshot(list_request)
                    .await
                    .map_err(|error| error.to_string())?;
                if list_response.status() != StatusCode::OK {
                    return Err(format!("invoice list returned {}", list_response.status()));
                }
                let body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
                    .await
                    .map_err(|error| error.to_string())?;
                let listed: Vec<InvoiceSummaryResponse> =
                    serde_json::from_slice(&body).map_err(|error| error.to_string())?;
                if (can_read && (listed.len() != 1 || listed[0].id != first_draft_id))
                    || (!can_read && !listed.is_empty())
                {
                    return Err("invoice list exposed or omitted a private invoice".to_owned());
                }
            }

            let stale_request = Request::builder()
                .method("POST")
                .uri(format!("/qr/{merchant_reference}/invoices"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::from(format!(
                    r#"{{"items":[{{"product_id":"{}","quantity":"0.333"}}],"expected_total":"532.80","idempotency_key":"stale-approval-{suffix}"}}"#,
                    product.id
                )))
                .map_err(|error| error.to_string())?;
            let stale_response = app(pool.clone())
                .oneshot(stale_request)
                .await
                .map_err(|error| error.to_string())?;
            if stale_response.status() != StatusCode::CONFLICT {
                return Err("a stale customer total created an invoice".to_owned());
            }

            let ordinary_reference: String =
                sqlx::query_scalar("SELECT account_qr_reference FROM users WHERE id = $1::uuid")
                    .bind(&admin_id)
                    .fetch_one(&pool)
                    .await
                    .map_err(|error| error.to_string())?;
            let ordinary_destination_request = Request::builder()
                .method("GET")
                .uri(format!("/qr/{ordinary_reference}"))
                .header("authorization", format!("Bearer {outsider_token}"))
                .body(Body::empty())
                .map_err(|error| error.to_string())?;
            let ordinary_destination_response = app(pool.clone())
                .oneshot(ordinary_destination_request)
                .await
                .map_err(|error| error.to_string())?;
            if ordinary_destination_response.status() != StatusCode::OK {
                return Err("customer QR was not resolved for another customer".to_owned());
            }
            let ordinary_body = axum::body::to_bytes(
                ordinary_destination_response.into_body(),
                usize::MAX,
            )
            .await
            .map_err(|error| error.to_string())?;
            let ordinary_destination: serde_json::Value =
                serde_json::from_slice(&ordinary_body).map_err(|error| error.to_string())?;
            if ordinary_destination["account_type"] != "PERSON_TRANSFER"
                || ordinary_destination["display_name"] != "Admin Test"
            {
                return Err("customer QR exposed the wrong destination".to_owned());
            }
            let self_destination_request = Request::builder()
                .method("GET")
                .uri(format!("/qr/{ordinary_reference}"))
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .map_err(|error| error.to_string())?;
            let self_destination_response = app(pool.clone())
                .oneshot(self_destination_request)
                .await
                .map_err(|error| error.to_string())?;
            if self_destination_response.status() != StatusCode::BAD_REQUEST {
                return Err("customer could use their own QR as a destination".to_owned());
            }
            let ordinary_catalog_request = Request::builder()
                .method("GET")
                .uri(format!("/qr/{ordinary_reference}/catalog"))
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .map_err(|error| error.to_string())?;
            let ordinary_catalog_response = app(pool.clone())
                .oneshot(ordinary_catalog_request)
                .await
                .map_err(|error| error.to_string())?;
            if ordinary_catalog_response.status() != StatusCode::NOT_FOUND {
                return Err("customer account exposed a merchant catalog".to_owned());
            }
            Ok(())
        }
        .await;

        sqlx::query("DELETE FROM invoices WHERE merchant_id = $1::uuid")
            .bind(&merchant_id)
            .execute(&pool)
            .await
            .expect("approval invoice cleanup should succeed");
        sqlx::query("DELETE FROM security_events WHERE user_id = $1::uuid")
            .bind(&admin_id)
            .execute(&pool)
            .await
            .expect("approval audit cleanup should succeed");
        sqlx::query("DELETE FROM users WHERE id::text IN ($1, $2, $3)")
            .bind(&admin_id)
            .bind(&applicant_id)
            .bind(&outsider_id)
            .execute(&pool)
            .await
            .expect("approval user cleanup should succeed");

        if let Err(error) = test_result {
            panic!("merchant approval integration test failed: {error}");
        }
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL and writes then removes isolated test data"]
    async fn merchant_application_stays_pending_and_cannot_be_duplicated() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await
            .expect("test database connection should succeed");
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let phone_number = format!("+2507{:010}", unique_suffix % 10_000_000_000);
        let password = "a secure merchant test password";

        let test_result: Result<(), String> = async {
            let register_request = Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"phone_number":"{phone_number}","password":"{password}","first_name":"Merchant"}}"#
                )))
                .map_err(|error| error.to_string())?;
            let register_response = app(pool.clone())
                .oneshot(register_request)
                .await
                .map_err(|error| error.to_string())?;
            if register_response.status() != StatusCode::CREATED {
                return Err(format!("registration returned {}", register_response.status()));
            }

            let login_request = Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"identifier":"{phone_number}","password":"{password}"}}"#
                )))
                .map_err(|error| error.to_string())?;
            let login_response = app(pool.clone())
                .oneshot(login_request)
                .await
                .map_err(|error| error.to_string())?;
            let token_body = axum::body::to_bytes(login_response.into_body(), usize::MAX)
                .await
                .map_err(|error| error.to_string())?;
            let token: LoginResponse =
                serde_json::from_slice(&token_body).map_err(|error| error.to_string())?;

            for expected_status in [StatusCode::CREATED, StatusCode::CONFLICT] {
                let application_request = Request::builder()
                    .method("POST")
                    .uri("/merchants/apply")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token.access_token))
                    .body(Body::from(r#"{"business_name":"Test Shop"}"#))
                    .map_err(|error| error.to_string())?;
                let response = app(pool.clone())
                    .oneshot(application_request)
                    .await
                    .map_err(|error| error.to_string())?;
                if response.status() != expected_status {
                    return Err(format!("application returned {}", response.status()));
                }
            }

            let application = sqlx::query_as::<_, (String, i64)>(
                "SELECT m.status, (SELECT COUNT(*)
                   FROM user_roles ur JOIN roles r ON r.id = ur.role_id
                   WHERE ur.user_id = u.id AND r.name = 'MERCHANT')
                 FROM merchant_profiles m JOIN users u ON u.id = m.user_id
                 WHERE u.phone_number = $1",
            )
            .bind(&phone_number)
            .fetch_one(&pool)
            .await
            .map_err(|error| error.to_string())?;
            if application != ("PENDING".to_owned(), 0) {
                return Err("application granted merchant access before approval".to_owned());
            }
            Ok(())
        }
        .await;

        sqlx::query("DELETE FROM security_events WHERE user_id = (SELECT id FROM users WHERE phone_number = $1)")
            .bind(&phone_number)
            .execute(&pool)
            .await
            .expect("security-event cleanup should succeed");
        sqlx::query("DELETE FROM users WHERE phone_number = $1")
            .bind(&phone_number)
            .execute(&pool)
            .await
            .expect("test user cleanup should succeed");

        if let Err(error) = test_result {
            panic!("merchant application integration test failed: {error}");
        }
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL and writes then removes isolated test data"]
    async fn login_and_logout_store_and_revoke_a_session() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await
            .expect("test database connection should succeed");
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let phone_number = format!("+2507{:010}", unique_suffix % 10_000_000_000);
        let email = format!("auth-test-{unique_suffix}@example.test");
        let password = "a secure integration test password";

        let test_result: Result<(), String> = async {
            let registration = Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"phone_number":"{phone_number}","email":"{email}","password":"{password}","first_name":"Integration"}}"#,
                )))
                .map_err(|error| error.to_string())?;
            let registration_response = app(pool.clone())
                .oneshot(registration)
                .await
                .map_err(|error| error.to_string())?;

            if registration_response.status() != StatusCode::CREATED {
                return Err(format!(
                    "registration returned {}",
                    registration_response.status()
                ));
            }

            let login = Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"identifier":"{phone_number}","password":"{password}"}}"#,
                )))
                .map_err(|error| error.to_string())?;
            let login_response = app(pool.clone())
                .oneshot(login)
                .await
                .map_err(|error| error.to_string())?;
            let login_status = login_response.status();
            let login_body = axum::body::to_bytes(login_response.into_body(), usize::MAX)
                .await
                .map_err(|error| error.to_string())?;

            if login_status != StatusCode::OK {
                return Err(format!("login returned {login_status}"));
            }

            let login_response: LoginResponse =
                serde_json::from_slice(&login_body).map_err(|error| error.to_string())?;
            let token_hash = hash_session_token(&login_response.access_token);
            let active_session = sqlx::query_scalar::<_, bool>(
                "SELECT revoked_at IS NULL
                 FROM user_sessions
                 WHERE token_hash = $1",
            )
            .bind(&token_hash)
            .fetch_one(&pool)
            .await
            .map_err(|error| error.to_string())?;

            if !active_session || login_response.token_type != "Bearer" {
                return Err("login did not create an active bearer session".to_owned());
            }

            let current_user = Request::builder()
                .method("GET")
                .uri("/auth/me")
                .header("authorization", format!("Bearer {}", login_response.access_token))
                .body(Body::empty())
                .map_err(|error| error.to_string())?;
            let current_user_response = app(pool.clone())
                .oneshot(current_user)
                .await
                .map_err(|error| error.to_string())?;
            let current_user_status = current_user_response.status();
            let current_user_body = axum::body::to_bytes(current_user_response.into_body(), usize::MAX)
                .await
                .map_err(|error| error.to_string())?;

            if current_user_status != StatusCode::OK {
                return Err(format!("current user returned {current_user_status}"));
            }

            let current_user: CurrentUserResponse =
                serde_json::from_slice(&current_user_body).map_err(|error| error.to_string())?;

            if current_user.phone_number != phone_number
                || current_user.email.as_deref() != Some(email.as_str())
                || current_user.first_name != "Integration"
                || current_user.roles != ["CUSTOMER"]
            {
                return Err("current user response did not match the authenticated user".to_owned());
            }

            let mut references = Vec::new();
            for _ in 0..2 {
                let request = Request::builder()
                    .method("GET")
                    .uri("/account/qr-reference")
                    .header("authorization", format!("Bearer {}", login_response.access_token))
                    .body(Body::empty())
                    .map_err(|error| error.to_string())?;
                let response = app(pool.clone())
                    .oneshot(request)
                    .await
                    .map_err(|error| error.to_string())?;
                if response.status() != StatusCode::OK {
                    return Err(format!("account QR reference returned {}", response.status()));
                }
                let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .map_err(|error| error.to_string())?;
                let reference: AccountQrReferenceResponse =
                    serde_json::from_slice(&body).map_err(|error| error.to_string())?;
                references.push(reference.reference);
            }
            if references[0] != references[1]
                || references[0].len() != 48
                || !references[0].bytes().all(|byte| byte.is_ascii_hexdigit())
                || references[0].contains(&phone_number)
            {
                return Err("account QR reference was not stable and opaque".to_owned());
            }

            let password_change = Request::builder()
                .method("PATCH")
                .uri("/auth/password")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", login_response.access_token))
                .body(Body::from(format!(
                    r#"{{"current_password":"{password}","new_password":"a secure changed integration password"}}"#,
                )))
                .map_err(|error| error.to_string())?;
            let password_change_response = app(pool.clone())
                .oneshot(password_change)
                .await
                .map_err(|error| error.to_string())?;

            if password_change_response.status() != StatusCode::NO_CONTENT {
                return Err(format!(
                    "password change returned {}",
                    password_change_response.status()
                ));
            }

            let logout = Request::builder()
                .method("POST")
                .uri("/auth/logout")
                .header("authorization", format!("Bearer {}", login_response.access_token))
                .body(Body::empty())
                .map_err(|error| error.to_string())?;
            let logout_response = app(pool.clone())
                .oneshot(logout)
                .await
                .map_err(|error| error.to_string())?;

            if logout_response.status() != StatusCode::NO_CONTENT {
                return Err(format!("logout returned {}", logout_response.status()));
            }

            let session_revoked = sqlx::query_scalar::<_, bool>(
                "SELECT revoked_at IS NOT NULL
                 FROM user_sessions
                 WHERE token_hash = $1",
            )
            .bind(token_hash)
            .fetch_one(&pool)
            .await
            .map_err(|error| error.to_string())?;

            if !session_revoked {
                return Err("logout did not revoke the session".to_owned());
            }

            let revoked_session = Request::builder()
                .method("GET")
                .uri("/auth/me")
                .header("authorization", format!("Bearer {}", login_response.access_token))
                .body(Body::empty())
                .map_err(|error| error.to_string())?;
            let revoked_session_response = app(pool.clone())
                .oneshot(revoked_session)
                .await
                .map_err(|error| error.to_string())?;

            if revoked_session_response.status() != StatusCode::UNAUTHORIZED {
                return Err("a revoked session was still accepted".to_owned());
            }

            Ok(())
        }
        .await;

        let identifier_hash = hash_identifier(&phone_number);

        sqlx::query(
            "DELETE FROM security_events
             WHERE user_id = (SELECT id FROM users WHERE phone_number = $1)
                OR identifier_hash = $2",
        )
        .bind(&phone_number)
        .bind(&identifier_hash)
        .execute(&pool)
        .await
        .expect("test security-event cleanup should succeed");

        sqlx::query("DELETE FROM users WHERE phone_number = $1")
            .bind(&phone_number)
            .execute(&pool)
            .await
            .expect("test user cleanup should succeed");

        if let Err(error) = test_result {
            panic!("login/logout integration test failed: {error}");
        }
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL and writes then removes isolated test data"]
    async fn failed_logins_are_rate_limited_and_audited() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await
            .expect("test database connection should succeed");
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let identifier = format!("rate-limit-{unique_suffix}@example.test");
        let identifier_hash = hash_identifier(&identifier);

        let test_result: Result<(), String> = async {
            for _ in 0..MAX_FAILED_LOGIN_ATTEMPTS {
                let request = Request::builder()
                    .method("POST")
                    .uri("/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"identifier":"{identifier}","password":"incorrect password"}}"#,
                    )))
                    .map_err(|error| error.to_string())?;
                let response = app(pool.clone())
                    .oneshot(request)
                    .await
                    .map_err(|error| error.to_string())?;

                if response.status() != StatusCode::UNAUTHORIZED {
                    return Err(format!("failed login returned {}", response.status()));
                }
            }

            let is_locked = sqlx::query_scalar::<_, bool>(
                "SELECT locked_until > NOW()
                 FROM login_rate_limits
                 WHERE identifier_hash = $1",
            )
            .bind(&identifier_hash)
            .fetch_one(&pool)
            .await
            .map_err(|error| error.to_string())?;

            if !is_locked {
                return Err("the fifth failed login did not lock the identifier".to_owned());
            }

            let failed_event_count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*)
                 FROM security_events
                 WHERE event_type = 'LOGIN_FAILED'
                   AND identifier_hash = $1",
            )
            .bind(&identifier_hash)
            .fetch_one(&pool)
            .await
            .map_err(|error| error.to_string())?;

            if failed_event_count != i64::from(MAX_FAILED_LOGIN_ATTEMPTS) {
                return Err(format!(
                    "expected {MAX_FAILED_LOGIN_ATTEMPTS} failed-login events, found {failed_event_count}"
                ));
            }

            let locked_request = Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"identifier":"{identifier}","password":"incorrect password"}}"#,
                )))
                .map_err(|error| error.to_string())?;
            let locked_response = app(pool.clone())
                .oneshot(locked_request)
                .await
                .map_err(|error| error.to_string())?;

            if locked_response.status() != StatusCode::UNAUTHORIZED {
                return Err(format!(
                    "rate-limited login returned {}",
                    locked_response.status()
                ));
            }

            let event_count_after_lock = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*)
                 FROM security_events
                 WHERE event_type = 'LOGIN_FAILED'
                   AND identifier_hash = $1",
            )
            .bind(&identifier_hash)
            .fetch_one(&pool)
            .await
            .map_err(|error| error.to_string())?;

            if event_count_after_lock != failed_event_count {
                return Err("a blocked request created an additional failure event".to_owned());
            }

            Ok(())
        }
        .await;

        sqlx::query("DELETE FROM security_events WHERE identifier_hash = $1")
            .bind(&identifier_hash)
            .execute(&pool)
            .await
            .expect("test security-event cleanup should succeed");
        sqlx::query("DELETE FROM login_rate_limits WHERE identifier_hash = $1")
            .bind(&identifier_hash)
            .execute(&pool)
            .await
            .expect("test rate-limit cleanup should succeed");

        if let Err(error) = test_result {
            panic!("login rate-limit integration test failed: {error}");
        }
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set before starting the IHUZA+ backend");
    let bind_address = env::var("APP_ADDRESS").unwrap_or_else(|_| "127.0.0.1:3000".to_owned());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    let app = app(pool);
    let listener = TcpListener::bind(&bind_address)
        .await
        .expect("Failed to bind the IHUZA+ backend listener");

    println!("IHUZA+ backend listening on http://{bind_address}");

    axum::serve(listener, app)
        .await
        .expect("IHUZA+ backend server failed");
}
