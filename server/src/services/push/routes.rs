use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::config;
use crate::error::{AppError, Result};
use crate::routes::AuthUser;
use crate::services::push::store;
use crate::services::push::store::Subscription;
use crate::state::AppState;

/// The key browsers subscribe against
#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "services/push.ts"))]
pub struct VapidKeyResponse {
    pub public_key: Vec<u8>
}

/// Names a subscription to remove
#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "services/push.ts"))]
pub struct UnsubscribeRequest {
    pub endpoint: String
}

// Public Methods //

/// Routes this service serves
pub fn router() -> Router<AppState> {
    if config::get().push.is_none() {
        return Router::new();
    }

    Router::new()
        .route("/push/vapid", get(vapid_key))
        .route("/me/push",
            post(subscribe)
            .delete(unsubscribe)
        )
}

/// Serves the key browsers subscribe against
pub async fn vapid_key(State(pool): State<SqlitePool>) -> Result<Json<VapidKeyResponse>> {

    let Some(keys) = store::keypair(&pool).await? else {
        return Err(AppError::NotFound);
    };

    let response = VapidKeyResponse {public_key: keys.public_key};
    Ok(Json(response))
}

/// Stores the caller's subscription
pub async fn subscribe(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Json(subscription): Json<Subscription>,
) -> Result<StatusCode> {
    store::save(&pool, user_id, &subscription).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Removes the caller's subscription
pub async fn unsubscribe(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Json(request): Json<UnsubscribeRequest>,
) -> Result<StatusCode> {
    store::remove(&pool, user_id, &request.endpoint).await?;
    Ok(StatusCode::NO_CONTENT)
}
