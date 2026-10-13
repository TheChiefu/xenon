pub mod accounts;
pub mod mappings;
pub mod microsoft;
pub mod presence;
pub mod routes;
pub mod session;
pub mod xsts;

use std::fmt;
use std::time::Duration;

use axum::routing::{get, post};
use axum::Router;

use crate::config;
use crate::services::xbox::mappings::Mappings;
use crate::state::AppState;

// Data Types //

/// Why a call to Microsoft or Xbox produced nothing usable
pub enum Failure {
    /// No later call for this account will succeed
    Dead(String),

    /// The service could not be reached or would not answer
    Retry(String)
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Failure::Dead(message) => write!(f, "{message}"),
            Failure::Retry(message) => write!(f, "{message}")
        }
    }
}

// Public Methods //

/// Routes this service serves
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/me/xbox",
            post(routes::start)
            .delete(routes::unlink)
        )
        .route("/xbox/redirect", get(routes::redirect))
}

/// Start presence polling and refresh token renewal, doing nothing when the
/// config has no xbox section
pub fn start(state: AppState) {
    let Some(xbox) = &config::get().xbox else {
        return;
    };

    let mappings = match xbox.game_mappings.is_empty() {
        true => Mappings::new(),
        false => mappings::load(&xbox.game_mappings)
    };

    tokio::spawn(poll_presence(state.clone(), mappings, xbox));
    tokio::spawn(renew_tokens(state, xbox));
}

// Helper Methods //

/// Read presence for every linked account, every interval
async fn poll_presence(
    state: AppState,
    mappings: Mappings,
    xbox: &config::Xbox,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(xbox.poll_interval_seconds));

    loop {
        interval.tick().await;
        presence::poll(&state, xbox, &mappings).await;
    }
}

/// Request a new refresh token for stale accounts, every interval
async fn renew_tokens(state: AppState, xbox: &config::Xbox) {
    let mut interval = tokio::time::interval(Duration::from_secs(xbox.refresh_interval_seconds));

    loop {
        interval.tick().await;
        session::sweep(&state.pool, xbox).await;
    }
}
