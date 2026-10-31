//! `user` commands
//!
//! Commands issued by the server maintainer(s) are trusted more than
//! HTTP calls over the internet.

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::api;
use crate::cli;
use crate::db;
use crate::error::{AppError, Result};
use crate::security;
use crate::services;
use crate::state::{self, AppState};
use crate::types::users::GlobalRole;
use crate::validate;

const COMMANDS: &[&str] = &[
    "user create <username>",
    "user role <username> <role>",
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
        ["create", username] => create(&state.pool, username).await,
        ["role", username, role] => set_role(&state.pool, username, role).await,
        ["unlock", username] => unlock(&state.pool, username).await,
        ["reset", username] => reset(&state.pool, username).await,
        ["get", username] => get(&state.pool, username).await,
        ["ban", username] => ban(state, username).await,
        ["unban", username] => unban(&state.pool, username).await,
        ["delete", username] => delete(state, username, false).await,
        ["delete", username, "--messages"] => delete(state, username, true).await,
        _ => Err(cli::usage(COMMANDS)),
    }
}

/// Creates an account with a generated password, printing it once
async fn create(pool: &SqlitePool, username: &str) -> Result<()> {
    validate::username(username)?;

    let password = security::generate_password();
    let hash = security::hash_password(&password)?;

    let mut conn = pool.acquire().await?;
    db::create_user(&mut conn, Uuid::now_v7(), username, username, &hash, GlobalRole::Visitor).await?;

    // Printed, never logged
    println!("created user account \"{username}\" with password: {password}");
    Ok(())
}

/// Sets an account's global role
async fn set_role(pool: &SqlitePool, username: &str, role: &str) -> Result<()> {
    let role: GlobalRole = role.parse()?;
    let user_id = get_user_id(pool, username).await?;

    let mut conn = pool.acquire().await?;
    db::set_global_role(&mut conn, user_id, role).await?;

    println!("set role of \"{username}\" to {role}");
    Ok(())
}

/// Bans an account
async fn ban(state: &AppState, username: &str) -> Result<()> {
    let user_id = get_user_id(&state.pool, username).await?;
    api::users::ban(&state.pool, user_id).await?;
    services::remove_user(state, user_id).await?;
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
    let rooms = api::users::delete(&state.pool, user_id, delete_history).await?;
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

    let password = security::generate_password();
    let hash = security::hash_password(&password)?;

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

// HELPER METHODS //

/// Look up an account's id by username
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
