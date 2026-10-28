//! HTTP handlers for registration, login, and registration codes.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::error::Result;
use crate::routes::{AuthUser, ClientIp};
use crate::state::AppState;
use crate::types::auth::{Registration, RegistrationCode};
use crate::{api, validate};

// REQUESTS //
#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct RequestRegistrationCreate {
    pub max_uses: Option<i64>,
    pub lifetime: Option<i64>,
    pub length: Option<usize>,
}

#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct RequestRegistrationRevoke {
    pub code: String,
}

#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct RequestLogin {
    pub username: String,
    pub password: String,
}

// RESPONSES //
#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct RespondRegistration {
    pub id: Uuid,
    pub session_token: String,
}

#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct RespondLogin {
    pub token: String,
}

#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct RespondRegistrationCode {
    pub code: String,
}


// ROUTING //

// Create an account
pub async fn register(
    ClientIp(ip): ClientIp,
    State(state): State<AppState>,
    Json(body): Json<Registration>,
) -> Result<(StatusCode, Json<RespondRegistration>)> {
    let (id, token) = api::auth::register(&state, ip, &body).await?;
    Ok((StatusCode::CREATED, Json(RespondRegistration { id, session_token: token })))
}

// Start a new session
pub async fn login(
    ClientIp(ip): ClientIp,
    State(state): State<AppState>,
    Json(body): Json<RequestLogin>,
) -> Result<Json<RespondLogin>> {
    let token = api::auth::sign_in(&state, ip, &body.username, &body.password).await?;
    Ok(Json(RespondLogin { token }))
}

// Create a code that lets someone register
pub async fn create_registration_code(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Json(body): Json<RequestRegistrationCreate>,
) -> Result<(StatusCode, Json<RespondRegistrationCode>)> {
    let max_uses = body.max_uses.unwrap_or(validate::REGISTRATION_CODE_DEFAULT_MAX_USES);
    let lifetime = body.lifetime.unwrap_or(validate::REGISTRATION_CODE_LIFETIME_MS);
    let length = body.length.unwrap_or(validate::REGISTRATION_CODE_DEFAULT_LENGTH);
    let code = api::auth::create_registration_code(
        &pool, caller_id, max_uses, lifetime, length
    ).await?;
    Ok((StatusCode::CREATED, Json(RespondRegistrationCode { code })))
}

// List every registration code
pub async fn list_registration_codes(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<RegistrationCode>>> {
    let codes = api::auth::list_registration_codes(&pool, caller_id).await?;
    Ok(Json(codes))
}

// Revoke a registration code
pub async fn revoke_registration_code(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Json(body): Json<RequestRegistrationRevoke>,
) -> Result<StatusCode> {
    api::auth::revoke_registration_code(&pool, caller_id, &body.code).await?;
    Ok(StatusCode::NO_CONTENT)
}
