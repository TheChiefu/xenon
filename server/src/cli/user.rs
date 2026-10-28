//! `user` commands
//! 
//! Separate from API calls to not mix responsiblities.
//! 
//! Commands issued from server maintainer(s) are assumed high trust level
//! than HTTP calls over the internet.

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::api;
use crate::error::{AppError, Result};
use crate::state::{self, AppState};
use crate::utils;

pub const COMMANDS: &[&str] = &[
    "user unlock <username>",
    "user reset <username>",
    "user get <username>",
    "user ban <username>",
    "user unban <username>",
    "user delete <username> [--messages]",
];

/// Runs the `user` command named by the arguments
pub async fn run(state: &AppState, args: &[&str]) -> Result<()> {
    match args {
        ["unlock", username] => unlock(&state.pool, username).await,
        ["reset", username] => reset(&state.pool, username).await,
        ["get", username] => get(&state.pool, username).await,
        ["ban", username] => ban(state, username).await,
        ["unban", username] => unban(&state.pool, username).await,
        ["delete", username] => delete(state, username, false).await,
        ["delete", username, "--messages"] => delete(state, username, true).await,
        _ => Err(super::usage(COMMANDS)),
    }
}

/// Bans an account
async fn ban(state: &AppState, username: &str) -> Result<()> {
    let user_id = get_user_id(&state.pool, username).await?;
    api::users::ban(state, user_id).await?;
    state::disconnect(state, user_id).await;

    println!("banned user account: \"{username}\"");
    Ok(())
}

/// Unbans an account
async fn unban(pool: &SqlitePool, username: &str) -> Result<()> {
    let user_id = get_user_id(pool, username).await?;
    api::users::unban(pool, user_id).await?;

    println!("unbanned user account: \"{username}\"");
    Ok(())
}

/// Deletes an account
async fn delete(state: &AppState, username: &str, delete_history: bool) -> Result<()> {
    let user_id = get_user_id(&state.pool, username).await?;
    let rooms = api::users::delete(state, user_id, delete_history).await?;
    state::disconnect(state, user_id).await;
    state::broadcast_member_left(state, user_id, rooms).await;

    println!("deleted user account: \"{username}\"");
    Ok(())
}

/// Lets a locked account log in again
async fn unlock(pool: &SqlitePool, username: &str) -> Result<()> {

    let user_id = get_user_id(pool, username).await?;
    let user = api::users::get(pool, user_id).await?;

    if !user.locked {
        return Err(AppError::Validation(format!("account \"{username}\" is not locked")));
    }

    // Unlock user account
    sqlx::query("UPDATE users SET locked = 0 WHERE id = ?1")
        .bind(user_id)
        .execute(pool)
        .await?;

    println!("unlocked user account: \"{username}\"");
    Ok(())
}

/// Replaces an account's password with a generated one, printing it once
async fn reset(pool: &SqlitePool, username: &str) -> Result<()> {
    let user_id = get_user_id(pool, username).await?;

    let password = utils::generate_code(12);
    let hash = utils::hash_password(&password)?;

    sqlx::query("UPDATE users SET password_hash = ?1 WHERE id = ?2")
        .bind(hash)
        .bind(user_id)
        .execute(pool)
        .await?;

    // Printed, never logged
    println!("new password for \"{username}\": {password}");
    Ok(())
}

/// Prints an account's details
async fn get(pool: &SqlitePool, username: &str) -> Result<()> {
    let user_id = get_user_id(pool, username).await?;
    let user = api::users::get(pool, user_id).await?;

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
        - Banned:\t{7}\n\
        +------------------------------------+\
        ",
        user.id, user.username, user.display_name, user.description,
        user.global_role, user.created_at, user.locked, user.banned
    );

    Ok(())
}

// Helper Methods //

/// Check if the given username exists
async fn get_user_id(pool: &SqlitePool, username: &str) -> Result<Uuid> {
    let id: Option<Uuid> = sqlx::query_scalar("
        SELECT id
        FROM users
        WHERE username = ?1
    ")
    .bind(username)
    .fetch_optional(pool)
    .await?;

    id.ok_or(AppError::Validation(format!("no account named \"{username}\"")))
}