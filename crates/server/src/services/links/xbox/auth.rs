//! Authorization headers for Xbox Live requests
//! - Caches one Authorization header per user until XSTS token expires
//! - Requests a new header with stored refresh token when none is cached

pub mod microsoft;
pub mod xsts;

use std::collections::BTreeMap;
use std::fmt;
use std::sync::RwLock;

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::config;
use crate::error::AppError;
use crate::services::links::xbox::accounts::{self, Account};
use crate::services::links::xbox::auth::xsts::XstsBody;
use crate::time::{DAY, HOUR, MINUTE, SECOND};
use crate::utils;

// DATA STRUCTS //

/// How to handle failures while authorizing an account:
/// - Dead (No calls for this account will succeed)
/// - Retry (A later attempt may succeed, such as when Microsoft or the database is down)
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

impl From<Failure> for AppError {
    fn from(failure: Failure) -> Self {
        match failure {
            Failure::Dead(message) => AppError::Validation(message),
            Failure::Retry(message) => AppError::Unavailable(message),
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

// PUBLIC METHODS //

/// Returns Authorization header for Xbox Live requests
pub async fn get_header(
    pool: &SqlitePool,
    account: &Account,
    xbox: &config::Xbox,
) -> Result<String, Failure> {

    // Returns cached header if not expired
    if let Some(value) = get_cached_header(account.user_id) {
        return Ok(value);
    }

    let xsts = request_xsts(pool, account, xbox).await?;

    // Check for expiry or invalid claims
    let Some(expires_at) = ms_rfc3339(&xsts.not_after) else {
        return Err(Failure::Retry("XSTS expiry was unreadable".into()));
    };

    // Finds the account's claims by XUID
    let mut claims = None;
    for entry in &xsts.display_claims.xui {
        if entry.xid == account.xuid {
            claims = Some(entry);
        }
    }
    let Some(claims) = claims else {
        return Err(Failure::Retry("XSTS response had no xui entry for this account".into()));
    };

    // Builds Authorization header from user hash and XSTS token, then cache it
    let value = format!("XBL3.0 x={};{}", claims.uhs, xsts.token);
    set_cached_header(account.user_id, &value, expires_at);

    Ok(value)
}

/// Removes cached Authorization header for a user
pub fn remove_header(user_id: Uuid) {
    utils::write_lock(&AUTHORIZATIONS).remove(&user_id);
}

// HELPER METHODS //

/// Requests XSTS token using stored refresh token
async fn request_xsts(
    pool: &SqlitePool,
    account: &Account,
    xbox: &config::Xbox,
) -> Result<XstsBody, Failure> {
    let tokens = microsoft::refresh_tokens(&account.refresh_token, xbox).await?;

    // Replaces stored refresh token with new one from response
    if let Some(refresh_token) = &tokens.refresh_token {
        if let Err(e) = accounts::set_refresh_token(pool, account.user_id, refresh_token).await {
            let message = format!("could not store a replacement refresh token: {e}");
            return Err(Failure::Retry(message));
        }
    }
    
    // Request Xbox Live user and XSTS tokens
    let user_token = xsts::request_user_token(&tokens.access_token).await?;
    xsts::request_xsts_token(&user_token).await
}

/// Caches Authorization header until it expires
fn set_cached_header(user_id: Uuid, value: &str, expires_at: i64) {
    let authorization = Authorization {
        value: value.to_string(),
        expires_at
    };
    utils::write_lock(&AUTHORIZATIONS).insert(user_id, authorization);
}

/// Returns cached Authorization header if not expired
fn get_cached_header(user_id: Uuid) -> Option<String> {
    let authorizations = utils::read_lock(&AUTHORIZATIONS);
    let authorization = authorizations.get(&user_id)?;

    match authorization.expires_at > utils::now_ms() {
        true => Some(authorization.value.clone()),
        false => None
    }
}

/// Read an RFC 3339 timestamp in UTC as milliseconds since the Unix epoch
fn ms_rfc3339(timestamp: &str) -> Option<i64> {
    const MONTH_START: [i64; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

    let (date, clock) = timestamp.split_once('T')?;

    let mut fields = date.split('-');
    let year: i64 = fields.next()?.parse().ok()?;
    let month: i64 = fields.next()?.parse().ok()?;
    let day: i64 = fields.next()?.parse().ok()?;

    // Catch invalid month and days
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let mut fields = clock.split(':');
    let hour: i64 = fields.next()?.parse().ok()?;
    let minute: i64 = fields.next()?.parse().ok()?;
    let second: i64 = fields.next()?.get(0..2)?.parse().ok()?;

    // Days into this year
    let mut days = MONTH_START[(month - 1) as usize] + day - 1;

    fn is_leap(year: i64) -> bool {
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }

    // Add a year of days for every year since 1970
    for past in 1970..year {
        days += 365;
        if is_leap(past) {
            days += 1;
        }
    }

    // Add this year's leap day once February has passed
    if month > 2 && is_leap(year) {
        days += 1;
    }

    Some(days * DAY + hour * HOUR + minute * MINUTE + second * SECOND)
}
