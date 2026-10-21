//! HTTP handlers for registration, login, and registration codes.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::error::{AppError, Result};
use crate::ratelimit::{AddressLimits, LoginFailures};
use crate::shared::{GlobalRole, RegistrationCode};
use crate::routes::{AuthUser, ClientIp};
use crate::{api, config, db, validate};

// Data Structs //

/// POST body for creating an account.
#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct RegisterRequest {
    pub registration_code: Option<String>,
    pub username: String,
    pub display_name: String,
    pub password: String,
}

/// POST body for starting a session.
#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// POST body for creating a registration code.
#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct CreateRegistrationCodeRequest {
    pub max_uses: Option<i64>,
    pub lifetime: Option<i64>,
    pub length: Option<usize>,
}

/// DELETE body for revoking a registration code.
#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct RevokeRegistrationCodeRequest {
    pub code: String,
}

/// Response carrying a new account's id and its first session token.
#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct RegisterResponse {
    pub id: Uuid,
    pub session_token: String,
}

/// Response carrying a session token.
#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct LoginResponse {
    pub token: String,
}

/// Response carrying a registration code
#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct CodeResponse {
    pub code: String,
}

// Routing Methods //

/// Creates an account.
///
/// # Arguments
///
/// * `pool` - Pool of SQL connections.
/// * `body` - Details for the new account.
pub async fn register(
    ClientIp(ip): ClientIp,
    State(address_limits): State<AddressLimits>,
    State(pool): State<SqlitePool>,
    Json(body): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>)> {

    let result = api::auth::register(
        &pool,
        body.registration_code.as_deref(),
        &body.username,
        &body.display_name,
        &body.password,
    ).await;

    // Count a failure caused by the client against its IP
    let failed = matches!(
        result, 
        Err(AppError::Validation(_) | AppError::UsernameTaken | AppError::InvalidRegistrationCode)
    );

    // If IP limit block is on, check limits to determine if block is necessary
    if failed && config::get().rate_limit.block_on_register_limit {
        if address_limits.register_limit_reached(ip) {
            address_limits.block(ip);
            let mut conn = pool.acquire().await?;
            db::save_blocked_address(&mut conn, ip, "registration").await?;
        }
    }

    let (id, token) = result?;

    Ok((StatusCode::CREATED, Json(RegisterResponse { id, session_token: token })))
}

/// Starts a session.
///
/// # Arguments
///
/// * `pool` - Pool of SQL connections.
/// * `body` - Credentials to authenticate with.
pub async fn login(
    ClientIp(ip): ClientIp,
    State(address_limits): State<AddressLimits>,
    State(login_failures): State<LoginFailures>,
    State(pool): State<SqlitePool>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>> {

    let result = api::auth::login(&pool, &login_failures, ip, &body.username, &body.password).await;

    // Count a wrong password, unknown username, or locked account against the IP
    let failed = matches!(result, Err(AppError::InvalidCredentials));

    // If IP limit block is on, check limits to determine if block is necessary
    if failed && config::get().rate_limit.block_on_login_limit {
        if address_limits.login_limit_reached(ip) {
            address_limits.block(ip);
            let mut conn = pool.acquire().await?;
            db::save_blocked_address(&mut conn, ip, "login").await?;
        }
    }

    let token = result?;

    Ok(Json(LoginResponse { token }))
}

/// Creates a code that lets someone register
pub async fn create_registration_code(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateRegistrationCodeRequest>,
) -> Result<(StatusCode, Json<CodeResponse>)> {

    let mut conn = pool.acquire().await?;

    // Check if user has permission to create a registration code
    let allowed = [GlobalRole::Admin];
    db::require_role(&mut conn, caller_id, &allowed).await?;

    // Create registration code
    let max_uses = body.max_uses.unwrap_or(validate::REGISTRATION_CODE_DEFAULT_MAX_USES);
    let lifetime = body.lifetime.unwrap_or(validate::REGISTRATION_CODE_LIFETIME_MS);
    let length = body.length.unwrap_or(validate::REGISTRATION_CODE_DEFAULT_LENGTH);
    validate::registration_code_params(max_uses, lifetime, length)?;
    let code = db::create_registration_code(
        &mut conn,
        caller_id,
        Some(max_uses),
        Some(lifetime),
        length
    ).await?;

    Ok((StatusCode::CREATED, Json(CodeResponse { code })))
}

/// Lists every registration code
pub async fn list_registration_codes(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<RegistrationCode>>> {

    let mut conn = pool.acquire().await?;

    let allowed = [GlobalRole::Admin];
    db::require_role(&mut conn, caller_id, &allowed).await?;
    drop(conn);

    let codes = api::auth::list_registration_codes(&pool).await?;

    Ok(Json(codes))
}

/// Revokes a registration code, deleting it outright
pub async fn revoke_registration_code(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Json(body): Json<RevokeRegistrationCodeRequest>,
) -> Result<StatusCode> {

    let mut conn = pool.acquire().await?;

    let allowed = [GlobalRole::Admin];
    db::require_role(&mut conn, caller_id, &allowed).await?;

    if !db::revoke_registration_code(&mut conn, &body.code.trim().to_ascii_uppercase()).await? {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
