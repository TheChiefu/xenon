//! The `rooms` table, and the tables scoped to a single room.

pub mod access;
pub mod bans;
pub mod invites;
pub mod read;

use sqlx::SqlitePool;
use sqlx::types::Json;
use uuid::Uuid;

use crate::db;
use crate::error::{AppError, Result};
use crate::types::rooms::{Permission, RoomInfo, Visibility};
use crate::types::users::GlobalRole;
use crate::utils;
use crate::validate;

// DATA STRUCTS //

/// A `rooms` row
#[derive(sqlx::FromRow)]
pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub visibility: Visibility,
    pub default_permissions: Json<Vec<Permission>>,
    pub created_at: i64,
}

impl From<Room> for RoomInfo {
    fn from(room: Room) -> Self {
        Self {
            id: room.id,
            name: room.name,
            visibility: room.visibility,
            default_permissions: room.default_permissions.0,
        }
    }
}

// PUBLIC METHODS //

/// Get a room's information by ID (hidden rooms only show to users in them)
pub async fn get(
    pool: &SqlitePool,
    room_id: Uuid,
    caller_id: Uuid,
) -> Result<Room> {

    let room: Room = sqlx::query_as(
        "
        SELECT r.id, r.name, r.visibility, r.default_permissions, r.created_at
        FROM rooms r
        WHERE r.id = ?1
            AND (r.visibility = ?2
            OR EXISTS(SELECT 1 FROM room_access a WHERE a.room_id = r.id AND a.user_id = ?3))
        "
    )
    .bind(room_id)
    .bind(Visibility::Public)
    .bind(caller_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(room)
}

/// Update a given room's properties
pub async fn update(
    pool: &SqlitePool,
    room_id: Uuid,
    caller_id: Uuid,
    name: Option<&str>,
    visibility: Option<Visibility>,
    default_permissions: Option<Vec<Permission>>,
) -> Result<()> {

    // Validate the new name, if one is given
    let renaming = name.is_some();
    let name = validate::room_name(name.unwrap_or(""))?;

    let changing_visibility = visibility.is_some();
    let granting = default_permissions.is_some();

    let mut tx = pool.begin_with(db::BEGIN_IMMEDIATE).await?;

    // Check if user has permissions
    let perms = db::room_permissions(&mut tx, room_id, caller_id).await?;
    let perms = perms.ok_or(AppError::Forbidden)?;

    // Check permissions for requested actions
    if (renaming || changing_visibility) && !perms.contains(&Permission::Rename) {
        return Err(AppError::Forbidden);
    }
    if granting && !perms.contains(&Permission::Grant) {
        return Err(AppError::Forbidden);
    }

    // - Flag decides whether the name is written
    // - NULL coalesced parameter leaves the columns as they are unchanged
    sqlx::query(
        "
        UPDATE rooms SET
            name = CASE WHEN ?1 THEN ?2 ELSE name END,
            visibility = COALESCE(?3, visibility),
            default_permissions = COALESCE(?4, default_permissions)
        WHERE id = ?5
        "
    )
    .bind(renaming)
    .bind(name)
    .bind(visibility)
    .bind(default_permissions.map(Json))
    .bind(room_id)
    .execute(&mut *tx)
    .await?;

    // Commit transaction
    tx.commit().await?;

    Ok(())
}

/// Create a room and return the new room id
pub async fn create(
    pool: &SqlitePool,
    caller_id: Uuid,
    name: &str,
    caller_permissions: Vec<Permission>,
    default_permissions: Vec<Permission>,
    visibility: Visibility,
) -> Result<Uuid> {

    let name = validate::room_name(name)?;

    // Open transaction
    let mut tx = pool.begin_with(db::BEGIN_IMMEDIATE).await?;

    // Check if user can create a room
    let allowed = [GlobalRole::Admin, GlobalRole::Member];
    db::require_role(&mut tx, caller_id, &allowed).await?;

    // Create room
    let now = utils::now_ms();
    let room_id = Uuid::now_v7();

    sqlx::query(
        "
        INSERT INTO rooms (id, name, visibility, default_permissions, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "
    )
    .bind(room_id)
    .bind(name)
    .bind(visibility)
    .bind(Json(default_permissions))
    .bind(now)
    .execute(&mut *tx)
    .await?;

    // Create room access
    sqlx::query(
        "
        INSERT INTO room_access (room_id, user_id, permissions, granted_at)
        VALUES (?1, ?2, ?3, ?4)
        "
    )
    .bind(room_id)
    .bind(caller_id)
    .bind(Json(caller_permissions))
    .bind(now)
    .execute(&mut *tx)
    .await?;

    // Start the creator's read position
    read::start(&mut tx, room_id, caller_id).await?;

    // If no errors, commit transaction
    tx.commit().await?;

    // Return room id
    Ok(room_id)
}

/// Delete a room, return member list to be used on notify
pub async fn delete(
    pool: &SqlitePool,
    room_id: Uuid,
    caller_id: Uuid,
) -> Result<Vec<Uuid>> {

    let mut tx = pool.begin_with(db::BEGIN_IMMEDIATE).await?;

    // Deleting a room requires Permission::DeleteRoom
    let perms = db::room_permissions(&mut tx, room_id, caller_id).await?;
    let permitted = perms.is_some_and(|p| p.contains(&Permission::DeleteRoom));
    let staff = db::staff_over_room(&mut tx, room_id, caller_id).await?;

    if !(permitted || staff) {
        return Err(AppError::Forbidden);
    }

    // Read the members before the delete
    let members = db::room_member_ids(&mut tx, room_id).await?;

    // Deleting the room also deletes its messages, attachments and other dependent rows
    sqlx::query("DELETE FROM rooms WHERE id = ?1")
    .bind(room_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(members)
}

/// Grant a user access to a room, spending their invite unless the room is public
pub async fn join(
    pool: &SqlitePool,
    room_id: Uuid,
    user_id: Uuid,
) -> Result<()> {

    // Open transaction
    let mut tx = pool.begin_with(db::BEGIN_IMMEDIATE).await?;

    // Check room visibility
    let visibility: Option<Visibility> = sqlx::query_scalar(
        "
        SELECT visibility
        FROM rooms
        WHERE id = ?1
        "
    )
    .bind(room_id)
    .fetch_optional(&mut *tx)
    .await?;

    let now = utils::now_ms();

    // Public is self service. Hidden requires an invite, and a
    // missing room is treated as closed
    let requires_invite = match visibility {
        Some(Visibility::Public) => false,
        Some(_) => true,
        None => return Err(AppError::Forbidden)
    };

    if requires_invite && !db::invited_to_room(&mut tx, room_id, user_id).await? {
        return Err(AppError::Forbidden);
    }

    if db::banned_from_room(&mut tx, room_id, user_id).await? {
        return Err(AppError::Forbidden);
    }

    // Grant user access
    sqlx::query(
        "
        INSERT INTO room_access (room_id, user_id, permissions, granted_at)
        SELECT ?1, ?2, r.default_permissions, ?3
        FROM rooms r
        WHERE r.id = ?1
        ON CONFLICT DO NOTHING
        "
    )
    .bind(room_id)
    .bind(user_id)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    // Start the user's read position
    read::start(&mut tx, room_id, user_id).await?;

    // Spend the invite the join was allowed on
    invites::delete(&mut tx, room_id, user_id).await?;

    // If no errors, commit transaction
    tx.commit().await?;

    Ok(())
}

/// Removes user from room, deleting the room once no members remain
pub async fn leave(
    pool: &SqlitePool,
    room_id: Uuid,
    user_id: Uuid,
) -> Result<()> {

    let mut tx = pool.begin_with(db::BEGIN_IMMEDIATE).await?;
    db::remove_from_room(&mut tx, room_id, user_id).await?;
    tx.commit().await?;

    Ok(())
}

/// Check whether a user may join a room's voice chat
pub async fn can_connect(
    pool: &SqlitePool,
    room_id: Uuid,
    user_id: Uuid,
) -> Result<bool> {

    let mut conn = pool.acquire().await?;
    let permissions = db::room_permissions(&mut conn, room_id, user_id).await?;

    // A user with no access to the room has no permissions
    match permissions {
        Some(permissions) => Ok(permissions.contains(&Permission::Connect)),
        None => Ok(false)
    }
}

/// List rooms a user is a member of
pub async fn list_mine(
    pool: &SqlitePool,
    user_id: Uuid,
) -> Result<Vec<Room>> {

    // Get list based on user's room access
    let rooms: Vec<Room> = sqlx::query_as(
        "
        SELECT r.id, r.name, r.visibility, r.default_permissions, r.created_at
        FROM rooms r JOIN room_access a ON a.room_id = r.id
        WHERE a.user_id = ?1
        ORDER BY r.id
        "
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rooms)
}

/// List one page of discoverable public rooms
pub async fn list_discoverable(
    pool: &SqlitePool,
    after: Option<Uuid>,
    limit: i64,
) -> Result<Vec<Room>> {

    let rooms: Vec<Room> = sqlx::query_as(
        "
        SELECT r.id, r.name, r.visibility, r.default_permissions, r.created_at
        FROM rooms r
        WHERE r.visibility = ?1
            AND (?2 IS NULL OR r.id > ?2)
        ORDER BY id
        LIMIT ?3
        "
    )
    .bind(Visibility::Public)
    .bind(after)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rooms)
}
