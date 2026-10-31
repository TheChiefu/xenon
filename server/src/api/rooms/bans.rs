//! The `room_bans` table: who is barred from a room.

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db;
use crate::error::{AppError, Result};
use crate::types::rooms::Permission;

// PUBLIC METHODS //

/// List the users banned from a room
pub async fn list(
    pool: &SqlitePool,
    room_id: Uuid,
    caller_id: Uuid,
) -> Result<Vec<Uuid>> {

    let mut conn = pool.acquire().await?;

    // Reading the list requires Permission::Ban
    require_permission(&mut conn, room_id, caller_id).await?;

    // Check for all users in ban list
    let user_ids: Vec<Uuid> = sqlx::query_scalar(
        "
        SELECT user_id
        FROM room_bans
        WHERE room_id = ?1
        "
    )
    .bind(room_id)
    .fetch_all(&mut *conn)
    .await?;

    Ok(user_ids)
}

/// Ban a user from a room and removes any membership they hold
pub async fn create(
    pool: &SqlitePool,
    room_id: Uuid,
    caller_id: Uuid,
    target_id: Uuid,
) -> Result<()> {

    let mut tx = pool.begin_with(db::BEGIN_IMMEDIATE).await?;

    require_permission(&mut tx, room_id, caller_id).await?;

    // Caller tries to ban themselves
    if caller_id == target_id {
        return Err(AppError::Validation("cannot ban yourself".to_string()));
    }

    // A user who can also ban cannot be banned
    let target = db::room_permissions(&mut tx, room_id, target_id).await?;
    if target.is_some_and(|p| p.contains(&Permission::Ban)) {
        return Err(AppError::Forbidden);
    }

    // Ban target user
    sqlx::query(
        "
        INSERT INTO room_bans (room_id, user_id)
        VALUES (?1, ?2)
        ON CONFLICT DO NOTHING
        "
    )
    .bind(room_id)
    .bind(target_id)
    .execute(&mut *tx)
    .await?;

    // Remove the target's membership. A user who is not in the room can still
    // be banned, so NotFound is discarded and every other error is returned
    match db::remove_from_room(&mut tx, room_id, target_id).await {
        Err(AppError::NotFound) => (),
        Err(err) => return Err(err),
        Ok(()) => ()
    }

    tx.commit().await?;

    Ok(())
}

/// Lift a user's ban on a room
pub async fn revoke(
    pool: &SqlitePool,
    room_id: Uuid,
    caller_id: Uuid,
    target_id: Uuid,
) -> Result<()> {

    let mut conn = pool.acquire().await?;

    require_permission(&mut conn, room_id, caller_id).await?;

    // Unban target user
    let result = sqlx::query("DELETE FROM room_bans WHERE room_id = ?1 AND user_id = ?2")
    .bind(room_id)
    .bind(target_id)
    .execute(&mut *conn)
    .await?;

    // No row deleted, so the user was not on the ban list
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}

// HELPER METHODS //

/// Reject the caller if they cannot issue bans for the room
async fn require_permission(
    conn: &mut sqlx::SqliteConnection,
    room_id: Uuid,
    caller_id: Uuid,
) -> Result<()> {

    let perms = db::room_permissions(&mut *conn, room_id, caller_id).await?;
    let permitted = perms.is_some_and(|p| p.contains(&Permission::Ban));
    let staff = db::staff_over_room(&mut *conn, room_id, caller_id).await?;

    if !(permitted || staff) {
        return Err(AppError::Forbidden);
    }

    Ok(())
}
