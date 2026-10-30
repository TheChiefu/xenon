//! Linking an Xbox account through Microsoft's OAuth sign-in

use std::collections::BTreeMap;
use std::sync::RwLock;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::config;
use crate::error::{AppError, Result};
use crate::services::xbox::accounts;
use crate::services::xbox::auth::{self, microsoft, xsts};
use crate::utils;

// STATE //

/// Code verifiers by user ID (PKCE code verifier for one sign-in)
static VERIFIERS: RwLock<BTreeMap<Uuid, String>> = RwLock::new(BTreeMap::new());

// PUBLIC METHODS //

/// Returns authorization request URL for a user
pub fn begin(user_id: Uuid) -> String {
    let xbox = config::get().xbox();

    // Random PKCE code verifier, kept until Microsoft redirects back
    let verifier = utils::generate_code(64);

    // SHA-256 hash of code verifier, sent as code challenge
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));

    // Replaces any earlier code verifier for this user
    utils::write_lock(&VERIFIERS).insert(user_id, verifier);

    microsoft::authorize_url(xbox, &user_id.to_string(), &challenge)
}

/// Redeems authorization code and saves linked account
pub async fn complete(
    pool: &SqlitePool,
    user_id: Uuid,
    code: Option<String>,
) -> Result<()> {
    let xbox = config::get().xbox();

    // Code verifier is removed so it can't be used twice
    let verifier = utils::write_lock(&VERIFIERS).remove(&user_id);

    // No code verifier means this server never started sign-in
    let Some(verifier) = verifier else {
        return Err(AppError::Forbidden);
    };

    // No authorization code when user cancels
    let Some(code) = code else {
        return Err(AppError::Validation("sign-in was not completed".into()));
    };

    // Microsoft checks code verifier against code challenge
    let tokens = microsoft::redeem_code(&code, &verifier, xbox).await?;

    // Rejects a token response without a refresh token
    let Some(refresh_token) = tokens.refresh_token else {
        return Err(AppError::Validation("Microsoft sent no refresh token".into()));
    };

    // Request user and XSTS token
    let user_token = xsts::request_user_token(&tokens.access_token).await?;
    let xsts = xsts::request_xsts_token(&user_token).await?;

    // Rejects an XSTS token response without display claims
    let Some(claims) = xsts.get_display_claims() else {
        return Err(AppError::Validation("XSTS response had no display claims".into()));
    };

    accounts::save(pool, user_id, &claims.xid, &claims.gtg, &refresh_token).await?;

    // Removes cached Authorization header for this user
    auth::remove_header(user_id);

    Ok(())
}
