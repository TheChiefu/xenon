//! Queries shared by more than one API module.
//!
//! Everything here takes a connection rather than a pool, so a caller can run
//! several of these inside one transaction.

use std::net::IpAddr;

use sqlx::types::Json;
use uuid::Uuid;

use crate::config;
use crate::error::{self, AppError, Result};
use crate::types::rooms::{Permission, Visibility};
use crate::types::time::DAY;
use crate::types::users::GlobalRole;
use crate::utils;

/// Creates a new user
pub async fn create_user(
    conn: &mut sqlx::SqliteConnection,
    id: Uuid,
    username: &str,
    display_name: &str,
    password_hash: &str,
    global_role: GlobalRole,
) -> Result<()> {

    sqlx::query(
        "
        INSERT INTO users (id, username, display_name, password_hash, global_role, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ",
    )
    .bind(id)
    .bind(username)
    .bind(display_name)
    .bind(password_hash)
    .bind(global_role)
    .bind(utils::now_ms())
    .execute(&mut *conn)
    .await
    .map_err(error::unique_violation)?;

    sqlx::query("INSERT INTO user_preferences (user_id) VALUES (?1)")
        .bind(id)
        .execute(&mut *conn)
        .await?;

    Ok(())
}

/// Opens a session for a user, returning the secret the client keeps
pub async fn create_session(
    conn: &mut sqlx::SqliteConnection,
    user_id: Uuid,
) -> Result<String> {

    let token = utils::generate_session_token();
    let lifetime = config::get().session.lifetime_days * DAY;
    sqlx::query(
        "
        INSERT INTO sessions (token_hash, user_id, created_at, expires_at)
        VALUES (?1, ?2, ?3, ?3 + ?4)
        "
    )
    .bind(token.hash.as_slice())
    .bind(user_id)
    .bind(utils::now_ms())
    .bind(lifetime)
    .execute(&mut *conn)
    .await?;

    Ok(token.secret)
}

/// Resolves what a user is allowed to do in a room
pub async fn effective_permissions(
    conn: &mut sqlx::SqliteConnection,
    room_id: Uuid,
    user_id: Uuid,
) -> Result<Option<Vec<Permission>>> {

    let stored: Option<Json<Vec<Permission>>> = sqlx::query_scalar(
        "
        SELECT a.permissions
        FROM room_access a
        WHERE a.room_id = ?1 AND a.user_id = ?2
        "
    )
    .bind(room_id)
    .bind(user_id)
    .fetch_optional(&mut *conn)
    .await?;

    match stored {
        Some(Json(permissions)) => Ok(Some(permissions)),
        None => Ok(None)
    }
}

/// Reads a user's server-wide role
pub async fn global_role(
    conn: &mut sqlx::SqliteConnection,
    user_id: Uuid,
) -> Result<GlobalRole> {

    let role: Option<GlobalRole> = sqlx::query_scalar(
        "SELECT global_role FROM users WHERE id = ?1"
    )
    .bind(user_id)
    .fetch_optional(&mut *conn)
    .await?;

    role.ok_or(AppError::NotFound)
}

/// Rejects a user whose global role is outside the allowed set
pub async fn require_role(
    conn: &mut sqlx::SqliteConnection,
    user_id: Uuid,
    allowed: &[GlobalRole],
) -> Result<()> {

    let role = global_role(&mut *conn, user_id).await?;
    if !allowed.contains(&role) {
        return Err(AppError::Forbidden);
    }

    Ok(())
}

/// Reports whether a user may act on a room.
/// Server Admins can act upon Public rooms.
pub async fn staff_over_room(
    conn: &mut sqlx::SqliteConnection,
    room_id: Uuid,
    user_id: Uuid,
) -> Result<bool> {

    let allowed = [GlobalRole::Admin];
    let role = global_role(&mut *conn, user_id).await?;
    if !allowed.contains(&role) {
        return Ok(false);
    }

    let public: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM rooms WHERE id = ?1 AND visibility = ?2)"
    )
    .bind(room_id)
    .bind(Visibility::Public)
    .fetch_one(&mut *conn)
    .await?;

    Ok(public)
}

/// Lists the ids of every member of a room
pub async fn room_member_ids(
    conn: &mut sqlx::SqliteConnection,
    room_id: Uuid,
) -> Result<Vec<Uuid>> {

    let members: Vec<Uuid> = sqlx::query_scalar(
        "
        SELECT user_id
        FROM room_access
        WHERE room_id = ?1
        "
    )
    .bind(room_id)
    .fetch_all(&mut *conn)
    .await?;

    Ok(members)
}

/// Lists the ids of every room a user is a member of
pub async fn member_room_ids(
    conn: &mut sqlx::SqliteConnection,
    user_id: Uuid,
) -> Result<Vec<Uuid>> {

    let rooms: Vec<Uuid> = sqlx::query_scalar(
        "
        SELECT room_id
        FROM room_access
        WHERE user_id = ?1
        "
    )
    .bind(user_id)
    .fetch_all(&mut *conn)
    .await?;

    Ok(rooms)
}

/// Lists the ids of everyone sharing a room with a user, the user included
pub async fn shared_room_member_ids(
    conn: &mut sqlx::SqliteConnection,
    user_id: Uuid,
) -> Result<Vec<Uuid>> {

    let members: Vec<Uuid> = sqlx::query_scalar(
        "
        SELECT DISTINCT user_id
        FROM room_access
        WHERE room_id IN (SELECT room_id FROM room_access WHERE user_id = ?1)
        "
    )
    .bind(user_id)
    .fetch_all(&mut *conn)
    .await?;

    Ok(members)
}

/// Writes a user's global role
pub async fn set_global_role(
    conn: &mut sqlx::SqliteConnection,
    user_id: Uuid,
    role: GlobalRole,
) -> Result<()> {

    sqlx::query(
        "
        UPDATE users SET global_role = ?1
        WHERE id = ?2
        "
    )
    .bind(role)
    .bind(user_id)
    .execute(&mut *conn)
    .await
    .map_err(error::unique_violation)?;

    Ok(())
}

// Get count of all users on server
pub async fn get_user_count(
    conn: &mut sqlx::SqliteConnection,
) -> Result<i64> {

    let count: i64 = sqlx::query_scalar(
        "
        SELECT COUNT(*)
        FROM users;
        "
    )
    .fetch_one(&mut *conn)
    .await?;

    Ok(count)
}

/// Get every blocked address
pub async fn get_blocked_addresses(
    conn: &mut sqlx::SqliteConnection,
) -> Result<Vec<IpAddr>> {

    let rows: Vec<String> = sqlx::query_scalar("
        SELECT address
        FROM blocked_addresses
    ")
    .fetch_all(&mut *conn)
    .await?;

    Ok(rows.iter().filter_map(|address| address.parse().ok()).collect())
}

/// Saves a blocked address, returning false if it was already saved
pub async fn save_blocked_address(
    conn: &mut sqlx::SqliteConnection,
    address: IpAddr,
    reason: &str,
) -> Result<bool> {

    let inserted = sqlx::query(
        "
        INSERT OR IGNORE INTO blocked_addresses (address, reason, created_at)
        VALUES (?1, ?2, ?3)
        "
    )
    .bind(address.to_string())
    .bind(reason)
    .bind(utils::now_ms())
    .execute(&mut *conn)
    .await?;

    Ok(inserted.rows_affected() > 0)
}