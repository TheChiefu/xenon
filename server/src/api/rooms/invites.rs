//! The `room_invites` table: who has been offered access to a room.

use uuid::Uuid;

use crate::db;
use crate::error::{AppError, Result};
use crate::types::rooms::Permission;

// Data Structs //

/// One invite a user has received, with the room's name joined in.
#[derive(sqlx::FromRow)]
pub struct Received {
    pub room_id: Uuid,
    pub room_name: Option<String>,
}

// API Methods //

/// Invites a user to a room.
///
/// # Arguments
///
/// * `pool` - Pool of SQL connections.
/// * `room_id` - Room being invited to.
/// * `caller_id` - Who is issuing the invite.
/// * `target_id` - Who is being invited.
pub async fn create(
    pool: &sqlx::SqlitePool,
    room_id: Uuid,
    caller_id: Uuid,
    target_id: Uuid,
) -> Result<()> {

    let mut tx = pool.begin_with(db::BEGIN_IMMEDIATE).await?;

    // Check if user has permission to create an invite
    require_permission(&mut tx, room_id, caller_id).await?;

    if caller_id == target_id {
        return Err(AppError::Validation("cannot invite yourself".to_string()));
    }

    if db::effective_permissions(&mut tx, room_id, target_id).await?.is_some() {
        return Err(AppError::Validation("user is already a member".to_string()));
    }

    if db::banned_from_room(&mut tx, room_id, target_id).await? {
        return Err(AppError::Validation("user is banned from this room".to_string()));
    }

    // Create invite
    sqlx::query(
        "
        INSERT INTO room_invites (room_id, user_id)
        VALUES (?1, ?2)
        ON CONFLICT DO NOTHING
        "
    )
    .bind(room_id)
    .bind(target_id)
    .execute(&mut *tx)
    .await?;

    // Commit transaction
    tx.commit().await?;

    Ok(())
}

/// Lists the invites addressed to a user, across every room.
///
/// # Arguments
///
/// * `pool` - Pool of SQL connections.
/// * `user_id` - User receiving the invites.
pub async fn list_for_user(
    pool: &sqlx::SqlitePool,
    user_id: Uuid,
) -> Result<Vec<Received>> {

    let mut conn = pool.acquire().await?;
    let invites: Vec<Received> = sqlx::query_as(
        "
        SELECT i.room_id, r.name AS room_name
        FROM room_invites i
        JOIN rooms r ON r.id = i.room_id
        WHERE i.user_id = ?1
        "
    )
    .bind(user_id)
    .fetch_all(&mut *conn)
    .await?;

    Ok(invites)
}


/// Lists the users a room has invited.
///
/// # Arguments
///
/// * `pool` - Pool of SQL connections.
/// * `room_id` - Room to query.
/// * `caller_id` - Who is requesting the list.
///
/// # Errors
///
/// Returns `AppError::Forbidden` if the caller lacks `Permission::Invite`.
pub async fn list(
    pool: &sqlx::SqlitePool,
    room_id: Uuid,
    caller_id: Uuid,
) -> Result<Vec<Uuid>> {

    let mut conn = pool.acquire().await?;

    // Reading the list requires Permission::Invite
    require_permission(&mut conn, room_id, caller_id).await?;

    let user_ids: Vec<Uuid> = sqlx::query_scalar(
        "
        SELECT user_id
        FROM room_invites
        WHERE room_id = ?1
        "
    )
    .bind(room_id)
    .fetch_all(&mut *conn)
    .await?;

    Ok(user_ids)
}

/// Declines an invite the caller received.
///
/// # Arguments
///
/// * `pool` - Pool of SQL connections.
/// * `room_id` - Room the invite is to.
/// * `caller_id` - Recipient of the invite.
///
/// # Errors
///
/// Returns `AppError::NotFound` if the caller holds no invite to the room.
pub async fn decline(
    pool: &sqlx::SqlitePool,
    room_id: Uuid,
    caller_id: Uuid,
) -> Result<()> {

    let mut conn = pool.acquire().await?;
    if !delete(&mut conn, room_id, caller_id).await? {
        return Err(AppError::NotFound);
    }

    Ok(())
}

/// Withdraws an invite the caller's room issued.
///
/// # Arguments
///
/// * `pool` - Pool of SQL connections.
/// * `room_id` - Room the invite is to.
/// * `caller_id` - Who is withdrawing the invite.
/// * `target_id` - User the invite was addressed to.
///
/// # Errors
///
/// Returns `AppError::Forbidden` if the caller lacks `Permission::Invite`, and
/// `AppError::NotFound` if the target holds no invite to the room.
pub async fn revoke(
    pool: &sqlx::SqlitePool,
    room_id: Uuid,
    caller_id: Uuid,
    target_id: Uuid,
) -> Result<()> {

    let mut conn = pool.acquire().await?;
    require_permission(&mut conn, room_id, caller_id).await?;

    if !delete(&mut conn, room_id, target_id).await? {
        return Err(AppError::NotFound);
    }

    Ok(())
}

/// Deletes a user's invite to a room, returning false when there was none.
///
/// # Arguments
///
/// * `conn` - Connection to SQL DB.
/// * `room_id` - Room the invite is to.
/// * `user_id` - Recipient of the invite.
pub async fn delete(
    conn: &mut sqlx::SqliteConnection,
    room_id: Uuid,
    user_id: Uuid,
) -> Result<bool> {

    let result = sqlx::query("DELETE FROM room_invites WHERE room_id = ?1 AND user_id = ?2")
    .bind(room_id)
    .bind(user_id)
    .execute(&mut *conn)
    .await?;

    Ok(result.rows_affected() > 0)
}

// Helper Methods //

/// Rejects the caller if they cannot issue invites for the room.
///
/// # Arguments
///
/// * `conn` - Connection to SQL DB.
/// * `room_id` - Room the permission applies to.
/// * `caller_id` - User being checked.
///
/// # Errors
///
/// Returns `AppError::Forbidden` if the caller lacks `Permission::Invite`.
async fn require_permission(
    conn: &mut sqlx::SqliteConnection,
    room_id: Uuid,
    caller_id: Uuid,
) -> Result<()> {

    let perms = db::effective_permissions(&mut *conn, room_id, caller_id).await?;
    if !perms.is_some_and(|p| p.contains(&Permission::Invite)) {
        return Err(AppError::Forbidden);
    }

    Ok(())
}
