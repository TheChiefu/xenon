//! Registration and login.

use std::net::IpAddr;

use uuid::Uuid;

use crate::config;
use crate::db;
use crate::error::{AppError, Result};
use crate::ratelimit::LoginFailures;
use crate::shared::{GlobalRole, RegistrationCode};
use crate::utils;
use crate::validate;

// API Methods //

/// Lists every registration code, newest first
///
/// # Arguments
///
/// * `pool` - Pool of SQL connections
pub async fn list_registration_codes(
    pool: &sqlx::SqlitePool,
) -> Result<Vec<RegistrationCode>> {

    let mut conn = pool.acquire().await?;

    let codes: Vec<RegistrationCode> = sqlx::query_as(
        "
        SELECT code, created_at, expires_at, max_uses, uses
        FROM registration_codes
        ORDER BY created_at DESC
        ",
    )
    .fetch_all(&mut *conn)
    .await?;

    Ok(codes)
}

/// Creates an account, returning its id and session token.
pub async fn register(
    pool: &sqlx::SqlitePool,
    code: Option<&str>,
    username: &str,
    display_name: &str,
    password: &str,
) -> Result<(Uuid, String)> {

    // Format Validation
    let username = username.trim();
    let display_name = display_name.trim();
    validate::username(username)?;
    validate::display_name(display_name)?;
    validate::password(password)?;

    let mut conn = pool.acquire().await?;

    // Reject a taken username before the registration code is claimed
    let taken: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM users WHERE username = ?1"
    )
    .bind(username)
    .fetch_optional(&mut *conn)
    .await?;

    if taken.is_some() {
        return Err(AppError::UsernameTaken);
    }

    // Claim registration code
    if config::get().session.registration_code_required {
        let code = code.ok_or(AppError::InvalidRegistrationCode)?;

        let claimed = sqlx::query(
            "
            UPDATE registration_codes SET uses = uses + 1
            WHERE code = ?1
                AND (expires_at IS NULL OR expires_at > ?2)
                AND (max_uses IS NULL OR uses < max_uses)
            ",
        )
        .bind(code.trim().to_ascii_uppercase())
        .bind(utils::now_ms())
        .execute(&mut *conn)
        .await?
        .rows_affected();

        if claimed == 0 {
            return Err(AppError::InvalidRegistrationCode);
        }
    }
    drop(conn);

    // Hash password for storage in DB
    let password_hash = utils::hash_password(password)?;

    // Open transaction
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    // Create user
    let id = Uuid::now_v7();
    db::create_user(
        &mut tx,
        id,
        username,
        display_name,
        &password_hash,
        GlobalRole::Visitor
    ).await?;

    // Create session key
    let token = db::create_session(&mut tx, id).await?;
    tx.commit().await?;

    Ok((id, token))
}

/// Replaces a user's password, optionally revoking the sessions they hold
/// elsewhere.
///
/// # Arguments
///
/// * `pool` - Pool of SQL connections.
/// * `user_id` - User whose password is being replaced.
/// * `current_password` - Password being replaced, re-verified before the write.
/// * `new_password` - Password to hash and store.
/// * `revoke_others` - Whether the user's other sessions are revoked.
/// * `session_hash` - `sessions.token_hash` of the caller, kept when revoking.
///
/// # Errors
///
/// Returns `AppError::Validation` if the new password is outside its length
/// limits, and `AppError::InvalidCredentials` if the current password does not
/// verify.
pub async fn change_password(
    pool: &sqlx::SqlitePool,
    user_id: Uuid,
    current_password: &str,
    new_password: &str,
    revoke_others: bool,
    session_hash: &[u8],
) -> Result<()> {

    validate::password(new_password)?;

    // Hash new password
    let password_hash = utils::hash_password(new_password)?;

    // Get stored password hash to check
    let stored_hash: Option<String> = sqlx::query_scalar(
        "
        SELECT password_hash
        FROM users
        WHERE id = ?1
        "
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    // No row: nothing to verify against
    let stored_hash = stored_hash.ok_or(AppError::InvalidCredentials)?;

    // Check if given current password matches the one stored in the DB
    if !utils::verify_password(current_password, &stored_hash)? {
        return Err(AppError::InvalidCredentials);
    }

    // Update password in DB
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    sqlx::query(
        "
        UPDATE users SET password_hash = ?1
        WHERE id = ?2
        "
    )
    .bind(password_hash)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    // If revocation is set, clear all other sessions except current one
    if revoke_others {
        sqlx::query(
            "
            UPDATE sessions SET revoked_at = ?1
            WHERE user_id = ?2 AND token_hash <> ?3 AND revoked_at IS NULL
            "
        )
        .bind(utils::now_ms())
        .bind(user_id)
        .bind(session_hash)
        .execute(&mut *tx)
        .await?;
    }

    // Commit transacation
    tx.commit().await?;

    Ok(())
}

/// Revokes a single session
pub async fn revoke_session(
    pool: &sqlx::SqlitePool,
    session_hash: &[u8],
) -> Result<()> {

    sqlx::query(
        "
        UPDATE sessions SET revoked_at = ?1
        WHERE token_hash = ?2 AND revoked_at IS NULL
        "
    )
    .bind(utils::now_ms())
    .bind(session_hash)
    .execute(pool)
    .await?;

    Ok(())
}

/// Verifies credentials and returns a new session token, locking the account after repeated failures
pub async fn login(
    pool: &sqlx::SqlitePool,
    failures: &LoginFailures,
    ip: IpAddr,
    username: &str,
    password: &str,
) -> Result<String> {

    #[derive(sqlx::FromRow)]
    struct LoginRow {id: Uuid, password_hash: String, locked: bool, banned: bool}

    // Username lookup
    let row: Option<LoginRow> = sqlx::query_as(
        "
        SELECT id, password_hash, locked, banned
        FROM users
        WHERE username = ?1
        "
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    // Not found, burn cycles to mask kind of API call
    let Some(user) = row else {
        utils::burn_verify(password);
        return Err(AppError::InvalidCredentials);
    };

    // Verify Password:
    // `?` is the corrupt-stored-hash case (server fault)
    // `false` is an incorrect password
    let verified = utils::verify_password(password, &user.password_hash)?;

    // Locked account returns the same error as a wrong password
    if user.locked {
        return Err(AppError::InvalidCredentials);
    }

    if !verified {

        // Lock the account once its failed logins reach the configured limit
        if failures.record_failure(user.id) {
            sqlx::query("UPDATE users SET locked = 1 WHERE id = ?1")
                .bind(user.id)
                .execute(pool)
                .await?;

            // Log IP on account lock
            tracing::warn!(
                "| [{ip}] account locked after failed logins [Username: {0} | ID: {1}]",
                username, user.id,
            );
        }

        return Err(AppError::InvalidCredentials);
    }

    if user.banned {
        return Err(AppError::Banned);
    }

    // Reset failure account once logged in
    failures.clear(user.id);

    // Create session key
    let mut conn = pool.acquire().await?;
    let token = db::create_session(&mut conn, user.id).await?;

    Ok(token)
}
