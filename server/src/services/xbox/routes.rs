use std::collections::BTreeMap;
use std::sync::RwLock;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::config;
use crate::error::{AppError, Result};
use crate::routes::AuthUser;
use crate::services::xbox;
use crate::services::xbox::accounts;
use crate::services::xbox::auth;
use crate::services::xbox::auth::microsoft;
use crate::services::xbox::auth::xsts;
use crate::state::AppState;
use crate::utils;

/// Page shown after linking, in tab client opened
const LINKED_PAGE: &str = r#"
<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <style>
    p, button {
    font-family: Noto Sans, sans-serif
    }
    html, body {
      height: 100%;
      margin: 0;
      background: rgba(0, 40, 100, 1);
    }
    body {
      display: flex;
      align-items: center;
      justify-content: center;
      font-family: sans-serif;
    }
    .box {
      border: 1px solid rgba(85, 85, 85, 1);
      border-radius: 8px;
      background: rgba(200, 200, 200, 1);
      padding: 24px;
      max-width: 400px;
      text-align: center;
    }
  </style>
</head>
<body>
  <div class="box">
    <p>Xbox account linked</p>
    <p>Click refresh icon on profile to update</p>
    <button onclick="window.close()">Return to  Client</button>
  </div>
</body>
</html>
"#;

// Data Types //

/// Code verifiers by user ID (PKCE code verifier for one sign-in)
static VERIFIERS: RwLock<BTreeMap<Uuid, String>> = RwLock::new(BTreeMap::new());

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

// Public Methods //

/// Returns authorization request URL for caller
pub async fn start(AuthUser(user_id, ..): AuthUser) -> Result<Json<LinkResponse>> {
    let xbox = config::get().xbox();

    // Random PKCE code verifier, kept until Microsoft redirects back
    let verifier = utils::generate_code(64);

    // SHA-256 hash of code verifier, sent as code challenge
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));

    // Replaces any earlier code verifier for this user
    utils::write_lock(&VERIFIERS).insert(user_id, verifier);

    let authorize_url = microsoft::authorize_url(xbox, &user_id.to_string(), &challenge);

    Ok(Json(LinkResponse {authorize_url}))
}

/// Saves linked account after Microsoft redirects back
/// - shows linked page on success
/// - shows error as plain text on failure
pub async fn redirect(
    State(pool): State<SqlitePool>,
    Query(query): Query<RedirectQuery>,
) -> Response {
    let user_id = query.state;

    match link(&pool, query).await {
        Ok(()) => Html(LINKED_PAGE).into_response(),
        Err(e) => {
            tracing::warn!("{user_id}: Xbox linking failed: {e}");
            let (status, message) = e.status_and_message();
            (status, format!("linking failed: {message}")).into_response()
        }
    }
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

// Helper Methods //

/// Redeems authorization code and saves linked account
async fn link(pool: &SqlitePool, query: RedirectQuery) -> Result<()> {
    let xbox = config::get().xbox();

    // Code verifier is removed so it can't be used twice
    let verifier = utils::write_lock(&VERIFIERS).remove(&query.state);

    // No code verifier means this server never started sign-in
    let Some(verifier) = verifier else {
        return Err(AppError::Forbidden);
    };

    // No authorization code when user cancels
    let Some(code) = query.code else {
        return Err(AppError::Validation("sign-in was not completed".into()));
    };

    // Microsoft checks code verifier against code challenge
    let tokens = microsoft::redeem_code(&code, &verifier, xbox).await?;

    // Rejects a token response without a refresh token
    let Some(refresh_token) = tokens.refresh_token else {
        return Err(AppError::Validation("Microsoft sent no refresh token".into()));
    };

    // Request user and XSTS token
    let user_token = xsts::request_user_token(&tokens.access_token).await?;
    let xsts = xsts::request_xsts_token(&user_token).await?;

    // Rejects an XSTS token response without display claims
    let Some(claims) = xsts.get_display_claims() else {
        return Err(AppError::Validation("XSTS response had no display claims".into()));
    };

    accounts::save(pool, query.state, &claims.xid, &claims.gtg, &refresh_token).await?;

    // Removes cached Authorization header for this user
    auth::remove_header(query.state);

    Ok(())
}
