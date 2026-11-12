//! Password hashing and session tokens

use std::sync::LazyLock;

use argon2::password_hash::rand_core::{OsRng, RngCore};
use argon2::password_hash::{self, PasswordHasher, SaltString};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use sha2::{Digest, Sha256};

use crate::config;
use crate::error::{AppError, Result};
use crate::utils;

/// How many bytes a session token carries
const SESSION_TOKEN_BYTES: usize = 32;

/// SHA-256 hash of a session token
pub type HashedToken = [u8; 32];

pub struct SessionToken {
    pub secret: String, // Goes to client (never stored on server)
    pub hash: HashedToken // Goes in sessions.token_hash (never leaves server)
}

// PASSWORD MANAGEMENT //

/// Generates a password within the configured length range
pub fn generate_password() -> String {
    const GENERATED_PASSWORD_LENGTH: usize = 12;
    let range = &config::get().lengths;
    let length = GENERATED_PASSWORD_LENGTH.clamp(range.password_min, range.password_max);
    utils::generate_code(length)
}

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
        Err(password_hash::Error::Password) => Ok(false),
        Err(e) => Err(AppError::Hash(e)),
    }
}

// SESSION MANAGEMENT //

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
pub fn hash_session_token(secret: &str) -> Option<HashedToken> {
    let bytes = hex::decode(secret).ok()?;
    if bytes.len() != SESSION_TOKEN_BYTES {
        return None;
    }
    Some(Sha256::digest(&bytes).into())
}

// SECURITY MANAGEMENT //

static DECOY_HASH: LazyLock<String> = LazyLock::new(|| {
    hash_password("decoy password, never matches a real login (hopefully)")
    .expect("hashing a fixed string cannot fail")
});

/// Burn CPU time a password verification costs, against a fixed hash
pub fn burn_verify(password: &str) {
    let _ = verify_password(password, &DECOY_HASH);
}
