//! The `users` table.

use std::collections::HashMap;

use uuid::Uuid;

use crate::db;
use crate::error::{AppError, Result};
use crate::services;
use crate::types::users::{GlobalRole, UserProfile};
use crate::utils;
use crate::validate;

// Data Structs //

// The `users` column
#[derive(sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub description: String,
    pub avatar_file_id: Option<Uuid>,
    pub banner_file_id: Option<Uuid>,
    pub global_role: GlobalRole,
    pub created_at: i64,
    pub locked: bool,
    pub banned: bool,
}

// The `users` columns a member row or search result shows
#[derive(sqlx::FromRow)]
pub struct UserSummary {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_file_id: Option<Uuid>,
    pub banner_file_id: Option<Uuid>,
    pub global_role: GlobalRole,
}


// API Methods //

// List one page of user summaries
pub async fn by_username_prefix(
    pool: &sqlx::SqlitePool,
    prefix: Option<String>,
    after: Option<Uuid>,
    limit: i64,
) -> Result<Vec<UserSummary>> {

    let mut conn = pool.acquire().await?;

    // Escape patterns for SQL string (optional user prefix)
    let pattern = prefix.map(|name| {
        let escaped = name
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        format!("{escaped}%")
    });

    let users: Vec<UserSummary> = sqlx::query_as(
        "
        SELECT id, username, display_name, avatar_file_id, banner_file_id, global_role
        FROM users u
        WHERE (?1 IS NULL OR u.id > ?1)
            AND (?2 IS NULL OR u.username LIKE ?2 ESCAPE '\\')
        ORDER BY id
        LIMIT ?3
        "
    )
    .bind(after)
    .bind(pattern)
    .bind(limit)
    .fetch_all(&mut *conn)
    .await?;

    Ok(users)
}

// Read a user's public profile by id
pub async fn get(
    pool: &sqlx::SqlitePool,
    user_id: Uuid,
) -> Result<User> {

    let mut conn = pool.acquire().await?;

    let row: Option<User> = sqlx::query_as(
        "
        SELECT id, username, display_name, description, avatar_file_id,
               banner_file_id, global_role, created_at, locked, banned
        FROM users
        WHERE id = ?1
        "
    )
    .bind(user_id)
    .fetch_optional(&mut *conn)
    .await?;

    row.ok_or(AppError::NotFound)
}

// Read a user's public profile with their linked accounts
pub async fn profile(
    pool: &sqlx::SqlitePool,
    user_id: Uuid,
) -> Result<UserProfile> {

    let row = get(pool, user_id).await?;
    let linked_accounts = services::list_linked_accounts(pool, user_id).await?;

    Ok(UserProfile {
        id: row.id,
        username: row.username,
        display_name: row.display_name,
        description: row.description,
        avatar_file_id: row.avatar_file_id,
        banner_file_id: row.banner_file_id,
        global_role: row.global_role,
        created_at: row.created_at,
        banned: row.banned,
        linked_accounts,
    })
}

// Read the summaries of a set of accounts
pub async fn by_ids(
    pool: &sqlx::SqlitePool,
    ids: &[Uuid],
) -> Result<Vec<UserSummary>> {

    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut conn = pool.acquire().await?;

    // Build SQL statement
    let mut builder = sqlx::QueryBuilder::new(
        "
        SELECT id, username, display_name, avatar_file_id, banner_file_id, global_role
        FROM users
        WHERE id IN (
        "
    );
    let mut list = builder.separated(", ");
    for id in ids {
        list.push_bind(*id);
    }
    list.push_unseparated(")");

    // Get list of summaries by given IDs
    let rows: Vec<UserSummary> = builder.build_query_as()
        .fetch_all(&mut *conn).await?;

    Ok(rows)
}

// Read the display names for a set of account IDs
pub async fn display_names(
    pool: &sqlx::SqlitePool,
    ids: &[Uuid],
) -> Result<HashMap<Uuid, String>> {

    if ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut conn = pool.acquire().await?;
    
    // Build SQL statement
    let mut builder = sqlx::QueryBuilder::new("SELECT id, display_name FROM users WHERE id IN (");
    let mut list = builder.separated(", ");
    for id in ids {
        list.push_bind(*id);
    }
    list.push_unseparated(")");

    // Get list of display names matching IDs
    let rows: Vec<(Uuid, String)> = builder.build_query_as().fetch_all(&mut *conn).await?;
    Ok(rows.into_iter().collect())
}

// Write a user's own profile, returning it as stored
// or `None` if no changes were made
pub async fn update(
    pool: &sqlx::SqlitePool,
    user_id: Uuid,
    display_name: Option<String>,
    description: Option<String>,
    avatar_file_id: Option<Uuid>,
    banner_file_id: Option<Uuid>,
) -> Result<Option<User>> {

    // Check if any field has changed from update
    let changed = display_name.is_some()
        || description.is_some()
        || avatar_file_id.is_some()
        || banner_file_id.is_some();

    if !changed {
        return Ok(None);
    }

    // Validate strings
    if let Some(name) = &display_name {
        validate::display_name(name)?;
    }
    if let Some(text) = &description {
        validate::profile_description(text)?;
    }

    let mut conn = pool.acquire().await?;

    // - COALESCE takes the sent value, or the column's own when none is sent
    // - nullif returns NULL when both arguments match, so the nil id clears
    // - RETURNING hands back what the row ends up holding, which is neither the
    //   value a patch left out nor the nil id it clears with
    let stored: User = sqlx::query_as(
        "
        UPDATE users SET
            display_name = COALESCE(?1, display_name),
            description = COALESCE(?2, description),
            avatar_file_id = nullif(COALESCE(?3, avatar_file_id), ?5),
            banner_file_id = nullif(COALESCE(?4, banner_file_id), ?5)
        WHERE id = ?6
        RETURNING id, username, display_name, description, avatar_file_id,
                  banner_file_id, global_role, created_at, locked, banned
        "
    )
    .bind(display_name.as_deref())
    .bind(description.as_deref())
    .bind(avatar_file_id)
    .bind(banner_file_id)
    .bind(Uuid::nil())
    .bind(user_id)
    .fetch_one(&mut *conn)
    .await?;

    Ok(Some(stored))
}

// Read a user's room list layout
pub async fn get_preferences(
    pool: &sqlx::SqlitePool,
    user_id: Uuid,
) -> Result<String> {

    let room_layout: String = sqlx::query_scalar(
        "
        SELECT room_layout
        FROM user_preferences
        WHERE user_id = ?1
        "
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(room_layout)
}

// Write a user's room list layout
pub async fn set_room_layout(
    pool: &sqlx::SqlitePool,
    user_id: Uuid,
    room_layout: &str,
) -> Result<()> {

    validate::room_layout(room_layout)?;

    sqlx::query(
        "
        UPDATE user_preferences SET room_layout = ?1
        WHERE user_id = ?2
        "
    )
    .bind(room_layout)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

// Delete an account, returning the rooms it left
pub async fn delete(
    pool: &sqlx::SqlitePool,
    user_id: Uuid,
    delete_history: bool,
) -> Result<Vec<Uuid>> {

    let mut tx = pool.begin_with(db::BEGIN_IMMEDIATE).await?;

    // Optionally delete the accounts message history
    if delete_history {
        delete_all_messages(&mut tx, user_id).await?;
    }

    // Leave every room
    let rooms = leave_every_room(&mut tx, user_id).await?;

    // Delete row from DB
    sqlx::query("DELETE FROM users WHERE id = ?1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    // Empty the WAL so no copies of deleted rows remain
    sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
    .execute(pool)
    .await?;

    Ok(rooms)
}

// Change a user's global role
pub async fn set_global_role(
    pool: &sqlx::SqlitePool,
    target_id: Uuid,
    role: GlobalRole,
) -> Result<()> {

    let mut conn = pool.acquire().await?;
    db::set_global_role(&mut conn, target_id, role).await?;

    Ok(())
}

// Ban a user
pub async fn ban(
    pool: &sqlx::SqlitePool,
    target_id: Uuid,
) -> Result<()> {

    let mut tx = pool.begin_with(db::BEGIN_IMMEDIATE).await?;

    // Update user's ban row
    sqlx::query("UPDATE users SET banned = 1 WHERE id = ?1")
        .bind(target_id)
        .execute(&mut *tx)
        .await?;

    // Revoke every session
    sqlx::query("
        UPDATE sessions
        SET revoked_at = ?1
        WHERE user_id = ?2 AND revoked_at IS NULL"
    )
    .bind(utils::now_ms())
    .bind(target_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

// Unban a user
pub async fn unban(
    pool: &sqlx::SqlitePool,
    target_id: Uuid,
) -> Result<()> {

    sqlx::query("UPDATE users SET banned = 0 WHERE id = ?1")
        .bind(target_id)
        .execute(pool)
        .await?;

    Ok(())
}

// Check whether a caller may moderate another user's account
pub async fn can_moderate(
    pool: &sqlx::SqlitePool,
    caller_id: Uuid,
    target_id: Uuid,
) -> Result<bool> {

    let mut conn = pool.acquire().await?;

    // Only Admins can moderate
    let caller_role = db::global_role(&mut conn, caller_id).await?;
    if caller_role != GlobalRole::Admin {
        return Ok(false);
    }

    // Admins can't be moderated
    let target_role = db::global_role(&mut conn, target_id).await?;
    Ok(target_role != GlobalRole::Admin)
}

// Helper Methods //

// Delete every message an account wrote
async fn delete_all_messages(
    conn: &mut sqlx::SqliteConnection,
    user_id: Uuid,
) -> Result<()> {

    // Attachments CASCADE
    sqlx::query("DELETE FROM messages WHERE author_id = ?1")
    .bind(user_id)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

// Remove an account from every room it belongs to
async fn leave_every_room(
    conn: &mut sqlx::SqliteConnection,
    user_id: Uuid,
) -> Result<Vec<Uuid>> {
    let rooms = db::member_room_ids(&mut *conn, user_id).await?;
    for room_id in &rooms {
        db::remove_from_room(&mut *conn, *room_id, user_id).await?;
    }

    // Return list of rooms remove from (to later notify members user left)
    Ok(rooms)
}
