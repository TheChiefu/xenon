// Steam account linking and presence polling
pub mod accounts;
pub mod api;
pub mod presence;
pub mod routes;

use std::time::Duration;

use axum::Router;
use axum::routing::{get, post};
use uuid::Uuid;

use crate::config;
use crate::error::Result;
use crate::state::platforms;
use crate::state::AppState;
use crate::types::platforms::Platform;

pub const REDIRECT_PATH: &str = "/steam/redirect";

pub fn router() -> Router<AppState> {
    if config::get().steam.is_none() {
        return Router::new();
    }

    Router::new()
        .route("/me/steam", 
            post(routes::start)
            .delete(routes::unlink)
        )
        .route(REDIRECT_PATH, get(routes::redirect))
}

pub async fn unlink(state: &AppState, user_id: Uuid) -> Result<()> {
    accounts::delete(&state.pool, user_id).await?;
    platforms::clear_metadata(state, user_id, Platform::Steam).await;

    Ok(())
}

pub fn start(state: AppState) {
    let Some(steam) = &config::get().steam else {
        return;
    };

    tokio::spawn(poll_presence(state, steam.poll_interval_seconds));
}

async fn poll_presence(state: AppState, poll_interval_seconds: u64) {
    let mut interval = tokio::time::interval(Duration::from_secs(poll_interval_seconds));

    loop {
        interval.tick().await;
        presence::poll(&state).await;
    }
}