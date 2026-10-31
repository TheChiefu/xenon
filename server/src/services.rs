//! Outside services the server links to, and the HTTP client they share

pub mod push;
pub mod steam;
pub mod xbox;

use std::sync::LazyLock;
use std::time::Duration;

use axum::Router;
use axum::response::{Html, IntoResponse, Response};
use reqwest::Client;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::config;
use crate::error::Result;
use crate::state::AppState;
use crate::types::messages::Notification;
use crate::types::platforms::Platform;
use crate::types::users::LinkedAccount;

/// Outbound HTTP client shared by every service
pub static CLIENT: LazyLock<Client> = LazyLock::new(build_client);

pub const PLATFORMS: &[Platform] = &[Platform::Steam, Platform::Xbox];

const LINKED_PAGE: &str = include_str!("services/linked_page.html");

// PUBLIC METHODS //

/// Respond to a platform's redirect with the linked page, or the reason linking failed
pub fn linked_response(platform: &str, user_id: Uuid, result: Result<()>) -> Response {
    match result {
        Ok(()) => Html(LINKED_PAGE).into_response(),
        Err(e) => {
            tracing::warn!("{user_id}: {platform} linking failed: {e}");
            let (status, message) = e.status_and_message();
            (status, format!("linking failed: {message}")).into_response()
        }
    }
}

/// Build outbound HTTP client at startup when config enables a service
pub fn init() {
    let config = config::get();

    if config.xbox.is_none()
        && config.push.is_none()
        && config.steam.is_none()
    {
        return;
    }

    LazyLock::force(&CLIENT);
}

/// Attempt to start each service (when applicable)
pub async fn start(state: &AppState) -> Result<()> {
    push::start(state).await?;

    for platform in PLATFORMS {
        match platform {
            Platform::Steam => steam::start(state.clone()),
            Platform::Xbox => xbox::start(state.clone()),
        }
    }

    Ok(())
}

/// Build a router of each service's routes (when enabled)
pub fn router() -> Router<AppState> {
    let mut router = push::routes::router();

    for platform in PLATFORMS {
        let platform_router = match platform {
            Platform::Steam => steam::routes::router(),
            Platform::Xbox => xbox::routes::router(),
        };

        router = router.merge(platform_router);
    }

    router
}

/// Read the accounts a user has linked on each platform
pub async fn list_linked_accounts(pool: &SqlitePool, user_id: Uuid) -> Result<Vec<LinkedAccount>> {
    let mut linked_accounts = Vec::new();

    for platform in PLATFORMS {
        let linked = match platform {
            Platform::Steam => steam::get_linked_account(pool, user_id).await?,
            Platform::Xbox => xbox::get_linked_account(pool, user_id).await?,
        };

        if let Some(linked) = linked {
            linked_accounts.push(linked);
        }
    }

    Ok(linked_accounts)
}

/// Notify the offline users among a notification's recipients
pub fn notify_offline(state: &AppState, notification: &Notification) {
    push::notify_offline(state, notification);
}

/// Remove a user's data from every service, enabled or not
pub async fn remove_user(state: &AppState, user_id: Uuid) -> Result<()> {
    push::remove_subscriptions(state, user_id).await?;

    for platform in PLATFORMS {
        match platform {
            Platform::Steam => steam::unlink(state, user_id).await?,
            Platform::Xbox => xbox::unlink(state, user_id).await?,
        }
    }

    Ok(())
}

// HELPER METHODS //

/// Build outbound HTTP client with request timeout, exits server if it cannot be built
fn build_client() -> Client {
    let timeout = Duration::from_secs(config::get().network.request_timeout_seconds);

    let built = Client::builder()
        .timeout(timeout)
        .build();

    match built {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("a service is enabled in config, but the outbound HTTP client could not be created: {e}");
            std::process::exit(1);
        }
    }
}
