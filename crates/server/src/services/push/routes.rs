//! HTTP handlers for push subscriptions

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use sqlx::SqlitePool;
use types::services::push::{
    Subscription,
    UnsubscribeRequest,
    VapidKeyResponse,
};

use crate::config;
use crate::error::{AppError, Result};
use crate::routes::AuthUser;
use crate::services::push::store;
use crate::state::AppState;

// ROUTING METHODS //

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
