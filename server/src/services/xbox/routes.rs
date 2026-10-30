//! HTTP handlers for Xbox account linking

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::config;
use crate::error::Result;
use crate::routes::AuthUser;
use crate::services;
use crate::services::xbox::{self, accounts, link, REDIRECT_PATH};
use crate::state::AppState;

// DATA STRUCTS //

/// Authorization request URL sent to client
#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "services/xbox.ts"))]
pub struct LinkResponse {
    pub authorize_url: String
}

/// Query string from Microsoft's redirect
/// - authorization code (absent when sign-in fails or is cancelled)
/// - user ID (sent as "state" in authorization request)
#[derive(Deserialize)]
pub struct RedirectQuery {
    code: Option<String>,
    state: Uuid
}

/// Caller's linked Xbox account
/// - Gamertag: Name shown for the account
/// - Honor cloaked: Whether activity is hidden while appearing offline on Xbox
#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "services/xbox.ts"))]
pub struct XboxResponse {
    pub gamertag: String,
    pub honor_cloaked: bool
}

/// Settings for caller's linked account
/// - Honor cloaked: Whether activity is hidden while appearing offline on Xbox
#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "services/xbox.ts"))]
pub struct XboxPatch {
    pub honor_cloaked: bool
}

// ROUTING METHODS //

/// Routes this service serves, none when config has no xbox section
pub fn router() -> Router<AppState> {
    if config::get().xbox.is_none() {
        return Router::new();
    }

    Router::new()
        .route("/me/xbox",
            get(get_account)
            .post(start)
            .patch(update)
            .delete(unlink)
        )
        .route(REDIRECT_PATH, get(redirect))
}

/// Returns authorization request URL for caller
pub async fn start(AuthUser(user_id, ..): AuthUser) -> Result<Json<LinkResponse>> {
    let authorize_url = link::begin(user_id);
    Ok(Json(LinkResponse {authorize_url}))
}

/// Saves linked account after Microsoft redirects back
/// - shows linked page on success
/// - shows error as plain text on failure
pub async fn redirect(
    State(pool): State<SqlitePool>,
    Query(query): Query<RedirectQuery>,
) -> Response {
    let result = link::complete(&pool, query.state, query.code).await;
    services::linked_response("Xbox", query.state, result)
}

/// Returns caller's linked account
pub async fn get_account(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
) -> Result<Json<Option<XboxResponse>>> {
    let response = match accounts::get(&pool, user_id).await? {
        Some(account) => Some(XboxResponse {
            gamertag: account.gamertag,
            honor_cloaked: account.honor_cloaked
        }),
        None => None
    };

    Ok(Json(response))
}

/// Changes caller's linked account settings
pub async fn update(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Json(body): Json<XboxPatch>,
) -> Result<StatusCode> {
    accounts::set_honor_cloaked(&pool, user_id, body.honor_cloaked).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Removes caller's linked account and cached header
pub async fn unlink(
    AuthUser(user_id, ..): AuthUser,
    State(app_state): State<AppState>,
) -> Result<StatusCode> {
    xbox::unlink(&app_state, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
