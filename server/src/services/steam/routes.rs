use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::config;
use crate::error::Result;
use crate::routes::AuthUser;
use crate::services::steam;
use crate::services::steam::{link, REDIRECT_PATH};
use crate::state::AppState;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

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
    <p>Account linked</p>
    <p>Click refresh icon on profile to update</p>
    <button onclick="window.close()">Return to  Client</button>
  </div>
</body>
</html>
"#;

// Data Types //

#[derive(Debug, Deserialize)]
pub struct RedirectQuery {
    user_id: Uuid,
    token: String,
    #[serde(flatten)]
    params: HashMap<String, String>
}

#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "services/steam.ts"))]
pub struct LinkResponse {
    pub login_url: String
}

// Public Methods //

pub fn router() -> Router<AppState> {
    if config::get().steam.is_none() {
        return Router::new();
    }

    Router::new()
        .route("/me/steam",
            post(start)
            .delete(unlink)
        )
        .route(REDIRECT_PATH, get(redirect))
}

pub async fn start(
    AuthUser(user_id, ..): AuthUser,
) -> Json<LinkResponse> {
    let login_url = link::begin(user_id);
    Json(LinkResponse { login_url })
}

pub async fn redirect(
    State(pool): State<SqlitePool>,
    Query(query): Query<RedirectQuery>,
) -> Response {
    match link::complete(&pool, query.user_id, &query.token, &query.params).await {
        Ok(()) => Html(LINKED_PAGE).into_response(),
        Err(e) => {
            tracing::warn!("Steam linking failed: {e}");
            let (status, message) = e.status_and_message();
            (status, format!("linking failed: {message}")).into_response()
        }
    }
}

pub async fn unlink(
    AuthUser(user_id, ..): AuthUser,
    State(state): State<AppState>,
) -> Result<StatusCode> {
    steam::unlink(&state, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
