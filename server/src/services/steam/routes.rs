//! HTTP handlers for Steam account linking

use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::config;
use crate::error::Result;
use crate::routes::AuthUser;
use crate::services;
use crate::services::steam::{self, link, REDIRECT_PATH};
use crate::state::AppState;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

// DATA STRUCTS //

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

// ROUTING METHODS //

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
    let result = link::complete(&pool, query.user_id, &query.token, &query.params).await;
    services::linked_response("Steam", query.user_id, result)
}

pub async fn unlink(
    AuthUser(user_id, ..): AuthUser,
    State(app_state): State<AppState>,
) -> Result<StatusCode> {
    steam::unlink(&app_state, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
