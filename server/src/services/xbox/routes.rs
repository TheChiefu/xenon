use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Html;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::config;
use crate::error::{AppError, Result};
use crate::routes::AuthUser;
use crate::services::xbox::accounts;
use crate::services::xbox::microsoft;
use crate::services::xbox::xsts;

/// Page the browser is left on once Microsoft redirects to it
const CLOSE_PAGE: &str = "<!doctype html><script>window.close()</script>";

// Data Types //

/// Where a user signs in to Microsoft
#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "services/xbox.ts"))]
pub struct LinkResponse {
    pub authorize_url: String
}

/// Query string Microsoft redirects with
#[derive(Deserialize)]
pub struct RedirectQuery {
    /// Authorization code Microsoft issues, absent when the sign-in did not
    /// finish
    code: Option<String>,

    /// User the sign-in belongs to
    state: Uuid
}

// Public Methods //

/// Serves the address the caller signs in to Microsoft at
pub async fn start(AuthUser(user_id, ..): AuthUser) -> Result<Json<LinkResponse>> {
    let Some(xbox) = &config::get().xbox else {
        return Err(AppError::Validation("account linking is unavailable".into()));
    };

    let authorize_url = microsoft::authorize_url(xbox, &user_id.to_string());

    Ok(Json(LinkResponse {authorize_url}))
}

/// Links the account a finished sign-in belongs to
pub async fn redirect(
    State(pool): State<SqlitePool>,
    Query(query): Query<RedirectQuery>,
) -> Result<Html<&'static str>> {
    let Some(xbox) = &config::get().xbox else {
        return Err(AppError::Validation("account linking is unavailable".into()));
    };

    let Some(code) = query.code else {
        return Err(AppError::Validation("sign-in was not completed".into()));
    };

    let tokens = match microsoft::exchange_code(&code, xbox).await {
        Ok(tokens) => tokens,
        Err(failure) => return Err(AppError::Validation(failure.to_string()))
    };

    let session = match xsts::session(&tokens.access_token).await {
        Ok(session) => session,
        Err(failure) => return Err(AppError::Validation(failure.to_string()))
    };

    let Some(refresh_token) = tokens.refresh_token else {
        return Err(AppError::Validation("Microsoft sent no refresh token".into()));
    };

    // Xbox lists one set of claims per signed-in account, and one is signed in
    let Some(claims) = session.display_claims.xui.into_iter().next() else {
        return Err(AppError::Validation("Xbox named no account".into()));
    };

    accounts::save(&pool, query.state, &claims.xid, &claims.gtg, &refresh_token).await?;

    Ok(Html(CLOSE_PAGE))
}

/// Removes the caller's linked account
pub async fn unlink(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
) -> Result<StatusCode> {
    accounts::remove(&pool, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
