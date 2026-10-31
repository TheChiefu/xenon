//! Utility data structures and methods

use std::panic::Location;
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use argon2::password_hash::rand_core::{OsRng, RngCore};

use crate::types::time::{DAY, HOUR, MINUTE, SECOND};

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
        let random = OsRng.next_u32() as usize;
        let index = random % CODE_ALPHABET.len();
        code.push(CODE_ALPHABET[index] as char);
    }
    code
}

/// Read an RFC 3339 timestamp in UTC as milliseconds since the Unix epoch
pub fn ms_rfc3339(timestamp: &str) -> Option<i64> {
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
