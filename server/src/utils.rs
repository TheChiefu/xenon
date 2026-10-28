//! Utility data structures and methods

use std::panic::Location;
use std::sync::{LazyLock, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use argon2::password_hash::rand_core::{OsRng, RngCore};
use argon2::password_hash::{Error as PhcError, PasswordHasher, SaltString};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use sha2::{Digest, Sha256};

use crate::error::{AppError, Result};
use crate::types::time::{DAY, HOUR, MINUTE, SECOND};

/// How many bytes a session token carries
const SESSION_TOKEN_BYTES: usize = 32;

pub struct SessionToken {
    pub secret: String, // Goes to client (never stored on server)
    pub hash: [u8; 32] // Goes in sessions.token_hash (never leaves server)
}


/// Return the current time in milliseconds since the Unix epoch
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before 1970")
        .as_millis() as i64
}

/// Generate a random code from uppercase alphanumerics
pub fn generate_code(length: usize) -> String {
    const CODE_ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

    let mut code = String::with_capacity(length);

    for _ in 0..length {
        let random = OsRng.next_u32() as usize; // Random Number from 0 to 4 billion
        let index = random % CODE_ALPHABET.len();   // Condense to range (0..36)
        code.push(CODE_ALPHABET[index] as char);    // Pick random char from allowed array
    }
    code
}

// Password Management //

/// Hash a password with Argon2id, returning a PHC string
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Report whether a password matches a stored hash
pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool> {
    let parsed = PasswordHash::new(stored_hash)?;

    match Argon2::default().verify_password(password.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(PhcError::Password) => Ok(false),
        Err(e) => Err(AppError::Hash(e)),
    }
}

// Session Management //

/// Generate a session token and its stored hash
pub fn generate_session_token() -> SessionToken {
    let mut bytes = [0u8; SESSION_TOKEN_BYTES];
    OsRng.fill_bytes(&mut bytes);

    SessionToken {
        hash: Sha256::digest(bytes).into(),
        secret: hex::encode(bytes)
    }
}

/// Hash a session token the client presented
pub fn hash_session_token(secret: &str) -> Option<[u8; 32]> {
    let bytes = hex::decode(secret).ok()?;
    if bytes.len() != SESSION_TOKEN_BYTES {
        return None;
    }
    Some(Sha256::digest(&bytes).into())
}

// Security Management //

static DECOY_HASH: LazyLock<String> = LazyLock::new(|| {
    hash_password("decoy password, never matches a real login (hopefully)")
    .expect("hashing a fixed string cannot fail")
});

/// Burn CPU time a password verification costs, against a fixed hash
pub fn burn_verify(password: &str) {
    let _ = verify_password(password, &DECOY_HASH);
}

/// Read an RFC 3339 timestamp in UTC as milliseconds since the Unix epoch
pub fn ms_rfc3339(timestamp: &str) -> Option<i64> {
    const MONTH_START: [i64; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

    let (date, clock) = timestamp.split_once('T')?;

    let mut fields = date.split('-');
    let year: i64 = fields.next()?.parse().ok()?;
    let month: i64 = fields.next()?.parse().ok()?;
    let day: i64 = fields.next()?.parse().ok()?;

    // Catch invalud month and days
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

// Locks

#[track_caller]
pub fn read_lock<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    match lock.read() {
        Ok(guard) => guard,
        Err(poisoned) => {
            tracing::warn!("lock was poisoned at {}, recovering", Location::caller());
            poisoned.into_inner()
        }
    }
}

#[track_caller]
pub fn write_lock<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    match lock.write() {
        Ok(guard) => guard,
        Err(poisoned) => {
            tracing::warn!("lock was poisoned at {}, recovering", Location::caller());
            poisoned.into_inner()
        }
    }
}

#[track_caller]
pub fn mutex_lock<T>(lock: &Mutex<T>) -> MutexGuard<'_, T> {
    match lock.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            tracing::warn!("lock was poisoned at {}, recovering", Location::caller());
            poisoned.into_inner()
        }
    }
}
