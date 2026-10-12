use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::error::{AppError, Result};
use crate::routes::AuthUser;
use crate::services::push::store;
use crate::services::push::store::Subscription;

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
