//! Utility data structures and methods

use std::panic::Location;
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use argon2::password_hash::rand_core::{OsRng, RngCore};

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
