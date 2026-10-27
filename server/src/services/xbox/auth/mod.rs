// Authorization headers for Xbox Live requests
// - Caches one Authorization header per user until XSTS token expires
// - Requests a new header with stored refresh token when none is cached
// - Removes linked account on invalid_grant from Microsoft or an XErr code from Xbox

pub mod microsoft;
pub mod xsts;

use std::collections::BTreeMap;
use std::fmt;
use std::sync::RwLock;

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::config;
use crate::services::xbox::accounts;
use crate::services::xbox::accounts::Account;
use crate::utils::{now_ms, ms_rfc3339, read_lock, write_lock};

// Data Types //

/// How to handle failures on Microsoft calls:
/// - Dead (No calls for this account will succeed)
/// - Retry (MS services may be down)
#[derive(Debug)]
pub enum Failure {
    Dead(String),
    Retry(String),
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Failure::Dead(message) | 
            Failure::Retry(message) => write!(f, "{message}")
        }
    }
}

/// Authorization header for one account's Xbox calls, with its expiry
struct Authorization {
    value: String,
    expires_at: i64
}

/// Authorization headers by user
type Authorizations = BTreeMap<Uuid, Authorization>;
static AUTHORIZATIONS: RwLock<Authorizations> = RwLock::new(BTreeMap::new());

// Public Methods //

/// Returns Authorization header for Xbox Live requests
pub async fn get_header(
    pool: &SqlitePool,
    account: &Account,
    xbox: &config::Xbox,
) -> Option<String> {

    // Returns cached header if not expired
    if let Some(value) = get_cached_header(account.user_id) {
        return Some(value);
    }

    // Requests and caches a new header
    match request_header(pool, account, xbox).await {

        // No errors, return header
        Ok(value) => Some(value),

        // If MS rejects refresh token, or Xbox rejects XSTS request with an XErr code
        Err(Failure::Dead(message)) => {

            // Remove the authorization header from the cache
            tracing::info!("{}: removing linked account ({message})", account.xuid);
            remove_header(account.user_id);

            // Attempt to remove a linked account from DB
            if let Err(e) = accounts::remove(pool, account.user_id).await {
                tracing::error!("{}: could not remove linked account: {e}", account.xuid);
            }
            None
        }

        // If failure is retryable, log warning
        Err(Failure::Retry(message)) => {
            tracing::warn!("{}: {message}", account.xuid);
            None
        }
    }
}

/// Removes cached Authorization header for a user
pub fn remove_header(user_id: Uuid) {
    write_lock(&AUTHORIZATIONS).remove(&user_id);
}

// Helper Methods //

/// Requests and caches a new Authorization header using stored refresh token
async fn request_header(
    pool: &SqlitePool,
    account: &Account,
    xbox: &config::Xbox,
) -> Result<String, Failure> {
    let tokens = microsoft::refresh_tokens(&account.refresh_token, xbox).await?;

    // Replaces stored refresh token with new one from response
    if let Some(refresh_token) = &tokens.refresh_token {
        accounts::set_refresh_token(pool, account.user_id, refresh_token)
            .await
            .map_err(|e| Failure::Retry(format!("could not store a replacement refresh token: {e}")))?;
    }

    // Request MS and XBL tokens
    let user_token = xsts::request_user_token(&tokens.access_token).await?;
    let xsts = xsts::request_xsts_token(&user_token).await?;

    // Check for expiry or invalid claims
    let Some(expires_at) = ms_rfc3339(&xsts.not_after) else {
        return Err(Failure::Retry("XSTS expiry was unreadable".into()));
    };
    let Some(claims) = xsts.get_display_claims() else {
        return Err(Failure::Retry("XSTS response had no display claims".into()));
    };

    // Builds Authorization header from user hash and XSTS token, then cache it
    let value = format!("XBL3.0 x={};{}", claims.uhs, xsts.token);
    write_lock(&AUTHORIZATIONS).insert(
        account.user_id,
        Authorization {
            value: value.clone(),
            expires_at
        }
    );

    Ok(value)
}

/// Returns cached Authorization header if not expired
fn get_cached_header(user_id: Uuid) -> Option<String> {
    let authorizations = read_lock(&AUTHORIZATIONS);
    let authorization = authorizations.get(&user_id)?;

    match authorization.expires_at > now_ms() {
        true => Some(authorization.value.clone()),
        false => None
    }
}
