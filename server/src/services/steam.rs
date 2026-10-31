//! Steam account linking and presence polling

pub mod accounts;
pub mod api;
pub mod link;
pub mod presence;
pub mod routes;

use std::time::Duration;

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::config;
use crate::error::Result;
use crate::state::platforms;
use crate::state::AppState;
use crate::types::platforms::Platform;
use crate::types::users::LinkedAccount;

pub const REDIRECT_PATH: &str = "/steam/redirect";

/// Read a user's linked Steam account
pub async fn get_linked_account(pool: &SqlitePool, user_id: Uuid) -> Result<Option<LinkedAccount>> {
    let Some(account) = accounts::get(pool, user_id).await? else {
        return Ok(None);
    };

    let linked = LinkedAccount {
        platform: Platform::Steam,
        id: account.steam_id,
        handle: account.persona_name
    };

    Ok(Some(linked))
}

pub async fn unlink(state: &AppState, user_id: Uuid) -> Result<()> {
    accounts::remove(&state.pool, user_id).await?;
    platforms::clear_metadata(state, user_id, Platform::Steam).await;

    Ok(())
}

pub fn start(state: AppState) {
    let Some(steam) = &config::get().steam else {
        return;
    };

    tracing::info!("enabled");

    tokio::spawn(poll_presence(state, steam.poll_interval_seconds));
}

async fn poll_presence(state: AppState, poll_interval_seconds: u64) {
    let mut interval = tokio::time::interval(Duration::from_secs(poll_interval_seconds));
    tracing::info!("polling at ({poll_interval_seconds} seconds)");

    loop {
        interval.tick().await;
        presence::poll(&state).await;
    }
}
