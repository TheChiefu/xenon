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
use crate::shared::GlobalRole;
use crate::routes::AuthUser;
use crate::{api, db, validate};

// Data Structs //

/// POST body for creating an account.
#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct RegisterRequest {
    pub registration_code: String,
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

/// Response carrying a registration code.
#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct CreateRegistrationCodeResponse {
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
    State(pool): State<SqlitePool>,
    Json(body): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>)> {

    let (id, token) = api::auth::register(
        &pool,
        &body.registration_code,
        &body.username,
        &body.display_name,
        &body.password,
    ).await?;

    Ok((StatusCode::CREATED, Json(RegisterResponse { id, session_token: token })))
}

/// Starts a session.
///
/// # Arguments
///
/// * `pool` - Pool of SQL connections.
/// * `body` - Credentials to authenticate with.
pub async fn login(
    State(pool): State<SqlitePool>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>> {

    let token = api::auth::login(&pool, &body.username, &body.password).await?;

    Ok(Json(LoginResponse { token }))
}

/// Creates a code that lets someone register
pub async fn create_registration_code(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateRegistrationCodeRequest>,
) -> Result<(StatusCode, Json<CreateRegistrationCodeResponse>)> {

    let mut conn = pool.acquire().await?;

    // Check if user has permission to create a registration code
    let allowed = [GlobalRole::Owner, GlobalRole::Admin];
    db::require_role(&mut conn, caller_id, &allowed).await?;

    // Create registration code
    let max_uses = body.max_uses.unwrap_or(validate::REGISTRATION_CODE_DEFAULT_MAX_USES);
    let lifetime = body.lifetime.unwrap_or(validate::REGISTRATION_CODE_LIFETIME_MS);
    validate::registration_code_params(max_uses, lifetime)?;
    let code = db::create_registration_code(&mut conn, caller_id, Some(max_uses), Some(lifetime)).await?;

    Ok((StatusCode::CREATED, Json(CreateRegistrationCodeResponse { code })))
}

/// Revokes a registration code, deleting it outright
pub async fn revoke_registration_code(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Json(body): Json<RevokeRegistrationCodeRequest>,
) -> Result<StatusCode> {

    let mut conn = pool.acquire().await?;

    let allowed = [GlobalRole::Owner, GlobalRole::Admin];
    db::require_role(&mut conn, caller_id, &allowed).await?;

    if !db::revoke_registration_code(&mut conn, &body.code.trim().to_ascii_uppercase()).await? {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
