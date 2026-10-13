use std::collections::BTreeMap;
use std::sync::RwLock;

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::config;
use crate::services::xbox::Failure;
use crate::services::xbox::accounts;
use crate::services::xbox::accounts::Account;
use crate::services::xbox::microsoft;
use crate::services::xbox::xsts;
use crate::utils::{now_ms, rfc3339_ms};

/// Days a refresh token is left unused before a new one is requested
const RENEW_AFTER_DAYS: i64 = 30;

// Data Types //

/// Authorization header for one account's Xbox calls, with its expiry
struct Authorization {
    value: String,
    expires_at: i64
}

/// Authorization headers by user
type Authorizations = BTreeMap<Uuid, Authorization>;
static AUTHORIZATIONS: RwLock<Authorizations> = RwLock::new(BTreeMap::new());

// Public Methods //

/// Authorization header for one account's Xbox calls
pub async fn authorization(
    pool: &SqlitePool,
    account: &Account,
    xbox: &config::Xbox,
) -> Option<String> {
    match get(account.user_id) {
        Some(value) => Some(value),
        None => requested(pool, account, xbox).await
    }
}

/// Request a new refresh token for every account whose stored one has gone
/// unused long enough for Microsoft to expire it
pub async fn sweep(pool: &SqlitePool, xbox: &config::Xbox) {
    let cutoff = now_ms() - RENEW_AFTER_DAYS * 24 * 60 * 60 * 1000;

    let stale = match accounts::stale(pool, cutoff).await {
        Ok(stale) => stale,
        Err(e) => {
            tracing::error!("could not read stale Xbox accounts: {e}");
            return;
        }
    };

    for account in stale {
        requested(pool, &account, xbox).await;
    }
}

// Helper Methods //

/// Send a refresh token to Microsoft for an access token, then that to Xbox
/// for an Authorization header
async fn requested(
    pool: &SqlitePool,
    account: &Account,
    xbox: &config::Xbox,
) -> Option<String> {
    let tokens = match microsoft::refresh_tokens(&account.refresh_token, xbox).await {
        Ok(tokens) => tokens,
        Err(Failure::Dead(message)) => {
            remove_account(pool, account, &message).await;
            return None;
        }
        Err(Failure::Retry(message)) => {
            tracing::warn!("{}: {message}", account.xuid);
            return None;
        }
    };

    // Microsoft stops accepting the refresh token it was sent, so its
    // replacement is stored before the access token beside it is used
    if !store(pool, account, &tokens).await {
        return None;
    }

    let session = match xsts::session(&tokens.access_token).await {
        Ok(session) => session,
        Err(Failure::Dead(message)) => {
            remove_account(pool, account, &message).await;
            return None;
        }
        Err(Failure::Retry(message)) => {
            tracing::warn!("{}: {message}", account.xuid);
            return None;
        }
    };

    let Some(expires_at) = rfc3339_ms(&session.not_after) else {
        tracing::warn!("{}: XSTS expiry was unreadable", account.xuid);
        return None;
    };

    // Xbox lists one set of claims per signed-in account, and one is signed in
    let Some(claims) = session.display_claims.xui.into_iter().next() else {
        tracing::warn!("{}: XSTS response named no account", account.xuid);
        return None;
    };

    let value = format!("XBL3.0 x={};{}", claims.uhs, session.token);
    insert(account.user_id, value.clone(), expires_at);

    Some(value)
}

/// Store a replacement refresh token, reporting whether it was stored
async fn store(pool: &SqlitePool, account: &Account, tokens: &microsoft::Tokens) -> bool {
    let Some(refresh_token) = &tokens.refresh_token else {
        return true;
    };

    match accounts::set_refresh_token(pool, account.user_id, refresh_token).await {
        Ok(()) => true,
        Err(e) => {
            tracing::error!("{}: could not store a replacement refresh token: {e}", account.xuid);
            false
        }
    }
}

/// Remove an account Microsoft or Xbox will not issue tokens for
async fn remove_account(pool: &SqlitePool, account: &Account, message: &str) {
    tracing::info!("{}: link is finished ({message}), removing", account.xuid);

    remove(account.user_id);

    match accounts::remove(pool, account.user_id).await {
        Ok(()) => (),
        Err(e) => tracing::error!("{}: could not remove a link: {e}", account.xuid)
    }
}

/// Read one account's header while Xbox still accepts it
fn get(user_id: Uuid) -> Option<String> {
    let authorizations = match AUTHORIZATIONS.read() {
        Ok(guard) => guard,
        Err(poisoned) => {
            tracing::warn!("xbox authorization lock was poisoned, recovering");
            poisoned.into_inner()
        }
    };

    let authorization = authorizations.get(&user_id)?;

    match authorization.expires_at > now_ms() {
        true => Some(authorization.value.clone()),
        false => None
    }
}

/// Replace one account's header
fn insert(user_id: Uuid, value: String, expires_at: i64) {
    let mut authorizations = match AUTHORIZATIONS.write() {
        Ok(guard) => guard,
        Err(poisoned) => {
            tracing::warn!("xbox authorization lock was poisoned, recovering");
            poisoned.into_inner()
        }
    };

    authorizations.insert(user_id, Authorization { value, expires_at });
}

/// Drop one account's header
fn remove(user_id: Uuid) {
    let mut authorizations = match AUTHORIZATIONS.write() {
        Ok(guard) => guard,
        Err(poisoned) => {
            tracing::warn!("xbox authorization lock was poisoned, recovering");
            poisoned.into_inner()
        }
    };

    authorizations.remove(&user_id);
}
