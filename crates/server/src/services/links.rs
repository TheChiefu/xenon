//! Platform accounts users link to their profile

pub mod platforms;
pub mod steam;
pub mod xbox;

use axum::Router;
use axum::response::{Html, IntoResponse, Response};
use sqlx::SqlitePool;
use uuid::Uuid;
use types::platforms::Platform;
use types::users::LinkedAccount;

use crate::config::{self, Config, Steam, Xbox};
use crate::error::Result;
use crate::state::AppState;

pub const PLATFORMS: &[Platform] = &[Platform::Steam, Platform::Xbox];

const LINKED_PAGE: &str = include_str!("links/linked_page.html");

impl Config {
    pub fn public_url(&self) -> &str {
        let url = self.network.public_url.as_deref().expect("network.public_url validated at startup");
        url.trim_end_matches('/')
    }

    /// Returns xbox section
    pub fn xbox(&self) -> &Xbox {
        self.xbox.as_ref().expect("xbox routes require xbox config")
    }

    /// Returns steam section
    pub fn steam(&self) -> &Steam {
        self.steam.as_ref().expect("steam routes require steam config")
    }
}

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

/// Whether config enables any platform
pub fn enabled() -> bool {
    let config = config::get();
    config.xbox.is_some() || config.steam.is_some()
}

/// Attempt to start each platform (when applicable)
pub fn start(state: &AppState) {
    for platform in PLATFORMS {
        match platform {
            Platform::Steam => steam::start(state.clone()),
            Platform::Xbox => xbox::start(state.clone()),
        }
    }
}

/// Build a router of each platform's routes (when enabled)
pub fn router() -> Router<AppState> {
    let mut router = Router::new();

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

/// Remove a user's linked account on every platform
pub async fn remove_user(state: &AppState, user_id: Uuid) -> Result<()> {
    for platform in PLATFORMS {
        match platform {
            Platform::Steam => steam::unlink(state, user_id).await?,
            Platform::Xbox => xbox::unlink(state, user_id).await?,
        }
    }

    Ok(())
}
