//! Xbox Live account linking and presence polling

pub mod accounts;
pub mod auth;
pub mod link;
pub mod presence;
pub mod routes;

use std::time::Duration;

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::config;
use crate::error::Result;
use crate::services::xbox::presence::mappings::{self, Mappings};
use crate::state::platforms;
use crate::state::AppState;
use crate::types::platforms::Platform;
use crate::types::users::LinkedAccount;

pub const REDIRECT_PATH: &str = "/xbox/redirect";

// PUBLIC METHODS //

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

/// Read a user's linked Xbox account
pub async fn get_linked_account(pool: &SqlitePool, user_id: Uuid) -> Result<Option<LinkedAccount>> {
    let Some(account) = accounts::get(pool, user_id).await? else {
        return Ok(None);
    };

    let linked = LinkedAccount {
        platform: Platform::Xbox,
        handle: account.gamertag
    };

    Ok(Some(linked))
}

/// Removes a user's linked account
pub async fn unlink(state: &AppState, user_id: Uuid) -> Result<()> {
    accounts::remove(&state.pool, user_id).await?;

    // Remove the cached header and the platform presence shown to others
    auth::remove_header(user_id);
    platforms::clear_metadata(state, user_id, Platform::Xbox).await;

    Ok(())
}

// HELPER METHODS //

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
