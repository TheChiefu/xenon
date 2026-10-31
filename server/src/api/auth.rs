//! Registration and login.

use std::net::IpAddr;

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::config;
use crate::db;
use crate::error::{AppError, Result};
use crate::ratelimit::{AddressLimits, LoginFailures};
use crate::security;
use crate::types::auth::{Registration, RegistrationCode};
use crate::types::time::DAY;
use crate::types::users::GlobalRole;
use crate::utils;
use crate::validate;

// PUBLIC METHODS //

/// Resolves a session to the user it belongs to, extending the session timer
pub async fn authenticate(
    pool: &SqlitePool,
    token_hash: &[u8],
) -> Result<Uuid> {

    let mut conn = pool.acquire().await?;

    // Fetch user id and session expiry information
    let now: i64 = utils::now_ms();
    let row = sqlx::query_as::<_, (Uuid, i64)>(
        "
        SELECT user_id, expires_at FROM sessions
        WHERE token_hash = ?1 AND revoked_at IS NULL AND expires_at > ?2
        "
    )
    .bind(token_hash)
    .bind(now)
    .fetch_optional(&mut *conn)
    .await?;

    let Some((user_id, expires_at)) = row else {
        return Err(AppError::InvalidCredentials);
    };

    // Extend session timer
    let lifetime = config::get().auth.lifetime_days * DAY;
    let renew = config::get().auth.renew_after_days_elapsed * DAY;
    if expires_at < now + lifetime - renew {
        sqlx::query(
            "
            UPDATE sessions SET expires_at = ?1 + ?2
            WHERE token_hash = ?3
            "
        )
        .bind(utils::now_ms())
        .bind(lifetime)
        .bind(token_hash)
        .execute(&mut *conn)
        .await?;
    }

    Ok(user_id)
}

/// Creates a code that lets someone register
pub async fn create_registration_code(
    pool: &SqlitePool,
    caller_id: Uuid,
    max_uses: i64,
    lifetime: i64,
    length: usize,
) -> Result<String> {

    let mut conn = pool.acquire().await?;

    // Check if user has permission to create a registration code
    let allowed = [GlobalRole::Admin];
    db::require_role(&mut conn, caller_id, &allowed).await?;

    validate::registration_code_params(max_uses, lifetime, length)?;

    let code = utils::generate_code(length);
    let now = utils::now_ms();

    sqlx::query(
        "
        INSERT INTO registration_codes (code, created_at, expires_at, max_uses)
        VALUES (?1, ?2, ?3, ?4)
        ",
    )
    .bind(&code)
    .bind(now)
    .bind(now.saturating_add(lifetime))
    .bind(max_uses)
    .execute(&mut *conn)
    .await?;

    Ok(code)
}

/// Revokes a registration code, deleting it outright
pub async fn revoke_registration_code(
    pool: &SqlitePool,
    caller_id: Uuid,
    code: &str,
) -> Result<()> {

    let mut conn = pool.acquire().await?;

    let allowed = [GlobalRole::Admin];
    db::require_role(&mut conn, caller_id, &allowed).await?;

    let deleted = sqlx::query(
        "
        DELETE FROM registration_codes
        WHERE code = ?1
        "
    )
    .bind(validate::registration_code(code))
    .execute(&mut *conn)
    .await?;

    if deleted.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}

/// Lists every registration code, newest first
pub async fn list_registration_codes(
    pool: &SqlitePool,
    caller_id: Uuid,
) -> Result<Vec<RegistrationCode>> {

    let mut conn = pool.acquire().await?;

    let allowed = [GlobalRole::Admin];
    db::require_role(&mut conn, caller_id, &allowed).await?;

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

/// Creates an account, counting a failed attempt against the IP
pub async fn register(
    pool: &SqlitePool,
    address_limits: &AddressLimits,
    ip: IpAddr,
    registration: &Registration,
) -> Result<(Uuid, String)> {

    let result = create_account(pool, registration).await;

    // Count a failure caused by the client against its IP
    let failed = matches!(
        result,
        Err(AppError::Validation(_) | AppError::UsernameTaken | AppError::InvalidRegistrationCode)
    );

    if failed && config::get().auth.block_on_register_limit {
        if address_limits.record_register_failure(ip) {
            block_address(pool, address_limits, ip, "registration").await?;
        }
    }

    result
}

/// Creates an account, returning its id and session token
async fn create_account(
    pool: &SqlitePool,
    registration: &Registration,
) -> Result<(Uuid, String)> {

    // Format Validation
    let username = registration.username.trim();
    let display_name = registration.display_name.trim();
    let password = registration.password.as_str();
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
    if config::get().auth.registration_code_required {
        let code = registration.registration_code.as_deref().ok_or(AppError::InvalidRegistrationCode)?;

        let claimed = sqlx::query(
            "
            UPDATE registration_codes SET uses = uses + 1
            WHERE code = ?1
                AND (expires_at IS NULL OR expires_at > ?2)
                AND (max_uses IS NULL OR uses < max_uses)
            ",
        )
        .bind(validate::registration_code(code))
        .bind(utils::now_ms())
        .execute(&mut *conn)
        .await?
        .rows_affected();

        if claimed == 0 {
            return Err(AppError::InvalidRegistrationCode);
        }
    }
    drop(conn);

    let password_hash = security::hash_password(password)?;

    // Open transaction
    let mut tx = pool.begin_with(db::BEGIN_IMMEDIATE).await?;

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

/// Replaces a user's password
pub async fn change_password(
    pool: &SqlitePool,
    user_id: Uuid,
    current_password: &str,
    new_password: &str,
) -> Result<()> {

    validate::password(new_password)?;

    let password_hash = security::hash_password(new_password)?;

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
    if !security::verify_password(current_password, &stored_hash)? {
        return Err(AppError::InvalidCredentials);
    }

    // Update password in DB
    let mut tx = pool.begin_with(db::BEGIN_IMMEDIATE).await?;
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

    tx.commit().await?;

    Ok(())
}

/// Revokes a single session
pub async fn revoke_session(
    pool: &SqlitePool,
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

/// Log a user in
pub async fn sign_in(
    pool: &SqlitePool,
    address_limits: &AddressLimits,
    login_failures: &LoginFailures,
    ip: IpAddr,
    username: &str,
    password: &str,
) -> Result<String> {

    let result = login(pool, login_failures, ip, username, password).await;

    // Count a failed login against the IP
    let failed = matches!(result, Err(AppError::InvalidCredentials));

    // Too many failures, block IP
    if failed && config::get().auth.block_on_login_limit {
        if address_limits.record_login_failure(ip) {
            block_address(pool, address_limits, ip, "login").await?;
        }
    }

    result
}

/// Verify credentials and return a new session token.
/// Lock account after repeated failures
async fn login(
    pool: &SqlitePool,
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
        security::burn_verify(password);
        return Err(AppError::InvalidCredentials);
    };

    let verified = security::verify_password(password, &user.password_hash)?;

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

    // Reset failure count once logged in
    failures.clear(user.id);

    // Create session key
    let mut conn = pool.acquire().await?;
    let token = db::create_session(&mut conn, user.id).await?;

    Ok(token)
}

async fn block_address(
    pool: &SqlitePool,
    address_limits: &AddressLimits,
    ip: IpAddr,
    reason: &str,
) -> Result<()> {

    address_limits.block(ip);

    let mut conn = pool.acquire().await?;
    db::save_blocked_address(&mut conn, ip, reason).await?;

    Ok(())
}
