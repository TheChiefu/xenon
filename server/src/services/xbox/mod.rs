// Xbox Live account linking and presence polling

pub mod accounts;
pub mod auth;
pub mod friends;
pub mod presence;
pub mod routes;

use std::time::Duration;

use axum::routing::get;
use axum::Router;
use uuid::Uuid;

use crate::config;
use crate::error::Result;
use crate::services::xbox::presence::mappings;
use crate::services::xbox::presence::mappings::Mappings;
use crate::shared::Platform;
use crate::sockets::chat::platforms;
use crate::state::AppState;

pub const REDIRECT_PATH: &str = "/xbox/redirect";

// Public Methods //

/// Routes this service serves, none when config has no xbox section
pub fn router() -> Router<AppState> {
    if config::get().xbox.is_none() {
        return Router::new();
    }

    Router::new()
        .route("/me/xbox",
            get(routes::get_account)
            .post(routes::start)
            .patch(routes::update)
            .delete(routes::unlink)
        )
        .route("/me/xbox/friends", get(routes::get_friends))
        .route(REDIRECT_PATH, get(routes::redirect))
}

/// Start presence polling, doing nothing when the config has no xbox section
pub fn start(state: AppState) {
    let Some(xbox) = &config::get().xbox else {
        return;
    };

    tracing::info!("enabled");

    let mappings = match xbox.game_mappings.is_empty() {
        true => Mappings::new(),
        false => mappings::load(&xbox.game_mappings)
    };

    tokio::spawn(poll_presence(state, mappings, xbox));
}

/// Removes a user's linked account
pub async fn unlink(state: &AppState, user_id: Uuid) -> Result<()> {
    accounts::remove(&state.pool, user_id).await?;

    // Remove the cached header and the game presence shown to others
    auth::remove_header(user_id);
    platforms::clear_presence(state, user_id, Platform::Xbox).await;

    Ok(())
}

// Helper Methods //

/// Polls presence for every linked account, every interval
async fn poll_presence(
    state: AppState,
    mappings: Mappings,
    xbox: &config::Xbox,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(xbox.poll_interval_seconds));
    tracing::info!("polling at ({} seconds)", xbox.poll_interval_seconds);

    loop {
        interval.tick().await;
        presence::poll(&state, xbox, &mappings).await;
    }
}
