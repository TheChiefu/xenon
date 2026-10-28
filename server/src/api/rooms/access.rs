//! The `room_access` table: who belongs to a room, and what they may do in it.

use sqlx::types::Json;
use uuid::Uuid;

use crate::db;
use crate::error::{AppError, Result};
use crate::types::rooms::{Notify, Permission};

// Data Structs //

#[derive(sqlx::FromRow)]
pub struct Entry {
    pub user_id: Uuid,
    pub permissions: Json<Vec<Permission>>,
    pub granted_at: i64,
    pub notify: Option<Notify>
}

#[derive(sqlx::FromRow)]
pub struct NotifyUserPair {
    pub user_id: Uuid,
    pub notify: Notify,
}

pub struct Access {
    pub permissions: Option<Vec<Permission>>,
    pub invited: bool,
    pub banned: bool,
}

// API Methods //

// Get a user's access state to a room
pub async fn get(
    pool: &sqlx::SqlitePool,
    room_id: Uuid,
    caller_id: Uuid,
    target_id: Uuid,
) -> Result<Access> {

    let mut conn = pool.acquire().await?;

    if db::effective_permissions(&mut conn, room_id, caller_id).await?.is_none() {
        return Err(AppError::Forbidden);
    }

    let permissions = db::effective_permissions(&mut conn, room_id, target_id).await?;
    let invited = db::invited_to_room(&mut conn, room_id, target_id).await?;
    let banned = db::banned_from_room(&mut conn, room_id, target_id).await?;

    Ok(Access { permissions, invited, banned })
}

// List the members' access entries in a room
pub async fn list(
    pool: &sqlx::SqlitePool,
    room_id: Uuid,
    caller_id: Uuid,
) -> Result<Vec<Entry>> {

    let mut conn = pool.acquire().await?;

    // Check if the caller is a member of the room
    if db::effective_permissions(&mut conn, room_id, caller_id).await?.is_none() {
        return Err(AppError::Forbidden);
    }

    // Get resulting vector of room members
    let result: Vec<Entry> = sqlx::query_as(
        "
        SELECT a.user_id, a.permissions, a.granted_at,
            CASE WHEN a.user_id = ?2 THEN a.notify END AS notify
        FROM room_access a
        WHERE a.room_id = ?1
        "
    )
    .bind(room_id)
    .bind(caller_id)
    .fetch_all(&mut *conn)
    .await?;

    Ok(result)
}

// Replace a member's permissions in a room
pub async fn update(
    pool: &sqlx::SqlitePool,
    room_id: Uuid,
    caller_id: Uuid,
    target_id: Uuid,
    permissions: Vec<Permission>,
) -> Result<()> {

    // One transaction, so the permissions read here cannot change before the write
    let mut tx = pool.begin_with(db::BEGIN_IMMEDIATE).await?;

    // Check caller's permission in the room
    let perms = db::effective_permissions(&mut tx, room_id, caller_id).await?;
    let Some(perms) = perms else {
        return Err(AppError::Forbidden);
    };

    if !perms.contains(&Permission::Grant) {
        return Err(AppError::Forbidden);
    }

    // A grant is bounded by the caller's own permissions
    if !permissions.iter().all(|p| perms.contains(p)) {
        let err = "cannot grant permissions you do not hold".to_string();
        return Err(AppError::Validation(err));
    }

    // Grant cannot be used against another holder. Your own row is exempt
    if caller_id != target_id {
        let target = db::effective_permissions(&mut tx, room_id, target_id).await?;
        let Some(target) = target else {
            return Err(AppError::NotFound);
        };

        if target.contains(&Permission::Grant) {
            return Err(AppError::Forbidden);
        }
    }

    // Change target's permissions
    sqlx::query(
        "
        UPDATE room_access
        SET permissions = ?1
        WHERE room_id = ?2 AND user_id = ?3
        "
    )
    .bind(Json(permissions))
    .bind(room_id)
    .bind(target_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

// Sets a user's notify level
pub async fn set_notify(
    pool: &sqlx::SqlitePool,
    room_id: Uuid,
    caller_id: Uuid,
    notify: Notify,
) -> Result<()> {

    let mut conn = pool.acquire().await?;

    let affected = sqlx::query(
        "
        UPDATE room_access
        SET notify = ?1
        WHERE room_id = ?2 AND user_id = ?3
        "
    )
    .bind(notify)
    .bind(room_id)
    .bind(caller_id)
    .execute(&mut *conn)
    .await?;

    if affected.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}

// List all notify and user ID pairs in a given room
pub async fn list_notify_pairs(
    pool: &sqlx::SqlitePool,
    room_id: Uuid,
) -> Result<Vec<NotifyUserPair>> {

    // Get resulting vector of room members
    let result: Vec<NotifyUserPair> = sqlx::query_as(
        "
        SELECT user_id, notify
        FROM room_access
        WHERE room_id = ?1
        "
    )
    .bind(room_id)
    .fetch_all(pool)
    .await?;

    Ok(result)
}

