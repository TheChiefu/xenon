//! `user` commands
//! 
//! Separate from API calls to not mix responsiblities.
//! 
//! Commands issued from server maintainer(s) are assumed high trust level
//! than HTTP calls over the internet.

use sqlx::SqlitePool;

use crate::api::users::User;
use crate::error::{AppError, Result};
use crate::state::AppState;
use crate::utils;

pub const COMMANDS: &[&str] = &[
    "user unlock <username>",
    "user reset <username>",
    "user get <username>",
];

/// Runs the `user` command named by the arguments
pub async fn run(state: &AppState, args: &[&str]) -> Result<()> {
    match args {
        ["unlock", username] => unlock(&state.pool, username).await,
        ["reset", username] => reset(&state.pool, username).await,
        ["get", username] => get(&state.pool, username).await,
        _ => Err(super::usage(COMMANDS)),
    }
}

/// Lets a locked account log in again
async fn unlock(pool: &SqlitePool, username: &str) -> Result<()> {

    // Determine if account exists
    let locked: Option<bool> = sqlx::query_scalar("SELECT locked FROM users WHERE username = ?1")
        .bind(username)
        .fetch_optional(pool)
        .await?;

    // Provide relevant messaging if account cannot be modified
    match locked {
        None => return Err(AppError::Validation(format!("no account named \"{username}\""))),
        Some(false) => return Err(AppError::Validation(format!("account \"{username}\" is not locked"))),
        Some(true) => {}
    }

    // Unlock user account
    sqlx::query("UPDATE users SET locked = 0 WHERE username = ?1")
        .bind(username)
        .execute(pool)
        .await?;

    println!("unlocked user account: \"{username}\"");
    Ok(())
}

/// Replaces an account's password with a generated one, printing it once
async fn reset(pool: &SqlitePool, username: &str) -> Result<()> {
    let password = utils::generate_code(12);
    let hash = utils::hash_password(&password)?;

    let updated = sqlx::query("UPDATE users SET password_hash = ?1 WHERE username = ?2 AND deleted_at IS NULL")
        .bind(hash)
        .bind(username)
        .execute(pool)
        .await?
        .rows_affected();

    if updated == 0 {
        return Err(AppError::Validation(format!("no account named \"{username}\"")));
    }

    // Printed, never logged
    println!("new password for \"{username}\": {password}");
    Ok(())
}

async fn get(pool: &SqlitePool, username: &str) -> Result<()> {

    let mut conn = pool.acquire().await?;
    let user: Option<User> = sqlx::query_as(
        "
        SELECT
            id, username, display_name, description, avatar_file_id,
            banner_file_id, global_role, created_at, locked, deleted_at
        FROM users
        WHERE username = ?1
        "
    )
    .bind(username)
    .fetch_optional(&mut *conn)
    .await?;

    match user {
        Some(user) => {
            println!(
                "+------------------------------------+\n\
                Selected User:\n\
                - ID:\t\t{0}\n\
                - Username:\t{1}\n\
                - Display Name:\t'{2}'\n\
                - Description:\t'{3}'\n\
                - Global Role:\t{4}\n\
                - Created At:\t{5}\n\
                - Locked:\t{6}\n\
                +------------------------------------+\
                ",
                user.id, user.username, user.display_name, user.description,
                user.global_role, user.created_at, user.locked
            );
            Ok(())
        }
        None => {
            return Err(AppError::Validation(format!("user not found")));
        }
    }
}