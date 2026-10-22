//! The `users` table.

use std::collections::HashMap;

use uuid::Uuid;

use crate::api::rooms::access;
use crate::db;
use crate::error::{AppError, Result};
use crate::shared::{GlobalRole};
use crate::validate;

// Data Structs //

/// The `users` column
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
}

/// The `users` columns a member row or search result shows
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

/// Lists one page of user summaries, optionally filtered by a username prefix.
///
/// # Arguments
///
/// * `pool` - Pool of SQL connections.
/// * `prefix` - Username prefix to match on, or `None` for every user.
/// * `after` - User id to page from, or `None` for the first page.
/// * `limit` - How many users to return.
pub async fn by_username_prefix(
    pool: &sqlx::SqlitePool,
    prefix: Option<String>,
    after: Option<Uuid>,
    limit: i64,
) -> Result<Vec<UserSummary>> {

    let mut conn = pool.acquire().await?;

    // Escape patterns for SQL string
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

/// Reads a user's public profile by id.
///
/// # Arguments
///
/// * `pool` - Pool of SQL connections.
/// * `user_id` - User to look up.
///
/// # Errors
///
/// Returns `AppError::NotFound` if no such user exists.
pub async fn get(
    pool: &sqlx::SqlitePool,
    user_id: Uuid,
) -> Result<User> {

    let mut conn = pool.acquire().await?;

    let row: Option<User> = sqlx::query_as(
        "
        SELECT id, username, display_name, description, avatar_file_id,
               banner_file_id, global_role, created_at, locked
        FROM users
        WHERE id = ?1
        "
    )
    .bind(user_id)
    .fetch_optional(&mut *conn)
    .await?;

    row.ok_or(AppError::NotFound)
}

/// Reads the summaries of a set of accounts.
///
/// An id naming no account is absent from the result.
///
/// # Arguments
///
/// * `pool` - Pool of SQL connections.
/// * `ids` - Accounts to read.
pub async fn by_ids(
    pool: &sqlx::SqlitePool,
    ids: &[Uuid],
) -> Result<Vec<UserSummary>> {

    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut conn = pool.acquire().await?;

    let mut builder = sqlx::QueryBuilder::new(
        "
        SELECT id, username, display_name, avatar_file_id, banner_file_id, global_role
        FROM users
        WHERE id IN (
        "
    );

    // Each push adds one placeholder and holds its value
    let mut list = builder.separated(", ");
    for id in ids {
        list.push_bind(*id);
    }
    list.push_unseparated(")");

    let rows: Vec<UserSummary> = builder.build_query_as().fetch_all(&mut *conn).await?;

    Ok(rows)
}

/// Reads the display names for a set of accounts.
///
/// An id naming no account is absent from the map.
///
/// # Arguments
///
/// * `pool` - Pool of SQL connections.
/// * `ids` - Accounts to read.
pub async fn display_names(
    pool: &sqlx::SqlitePool,
    ids: &[Uuid],
) -> Result<HashMap<Uuid, String>> {

    if ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut conn = pool.acquire().await?;

    let mut builder = sqlx::QueryBuilder::new("SELECT id, display_name FROM users WHERE id IN (");

    // Each push adds one placeholder and holds its value
    let mut list = builder.separated(", ");
    for id in ids {
        list.push_bind(*id);
    }
    list.push_unseparated(")");

    let rows: Vec<(Uuid, String)> = builder.build_query_as().fetch_all(&mut *conn).await?;

    Ok(rows.into_iter().collect())
}

/// Writes a user's own profile, returning it as stored, or `None` if the patch
/// asked for no change.
///
/// # Arguments
///
/// * `pool` - Pool of SQL connections.
/// * `user_id` - User whose profile is being written.
/// * `patch` - Fields to change; an absent field is left unchanged.
///
/// # Errors
///
/// Returns `AppError::Validation` if the display name or the description is
/// outside its length limits.
pub async fn update(
    pool: &sqlx::SqlitePool,
    user_id: Uuid,
    display_name: Option<String>,
    description: Option<String>,
    avatar_file_id: Option<Uuid>,
    banner_file_id: Option<Uuid>,
) -> Result<Option<User>> {

    // Check if any field has changed from update patch
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
                  banner_file_id, global_role, created_at, locked
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

/// Deletes an account's row, leaving any kept messages without an author
pub async fn delete(
    pool: &sqlx::SqlitePool,
    user_id: Uuid,
    delete_history: bool,
) -> Result<Vec<Uuid>> {

    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    if delete_history {
        delete_messages(&mut tx, user_id).await?;
    }

    let rooms = leave_every_room(&mut tx, user_id).await?;

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

/// Closes someone else's account
/// 
/// Returns `Forbidden` if the caller is not Admin, or if an Admin targets another Admin
pub async fn delete_other(
    pool: &sqlx::SqlitePool,
    caller_id: Uuid,
    target_id: Uuid,
    delete_history: bool,
) -> Result<Vec<Uuid>> {

    let mut conn = pool.acquire().await?;

    // Is the caller ranked high enough to close another account
    let caller_role = db::global_role(&mut conn, caller_id).await?;
    let permissible_roles = [GlobalRole::Admin];
    if !permissible_roles.contains(&caller_role) {
        return Err(AppError::Forbidden);
    }

    // Admins can't close each other's accounts
    let target_role = db::global_role(&mut conn, target_id).await?;
    if caller_role == GlobalRole::Admin && target_role == GlobalRole::Admin {
        return Err(AppError::Forbidden);
    }

    delete(pool, target_id, delete_history).await
}

/// Changes a user's server-wide role.
///
/// # Arguments
///
/// * `pool` - Pool of SQL connections.
/// * `caller_id` - User performing the role change.
/// * `target_id` - User receiving the role change.
/// * `role` - Role the target receives.
///
/// # Errors
///
/// Returns `AppError::Forbidden` if the caller is not Admin, or if an Admin
/// targets another Admin; and `AppError::NotFound` if the target does not exist.
pub async fn set_role(
    pool: &sqlx::SqlitePool,
    caller_id: Uuid,
    target_id: Uuid,
    role: GlobalRole,
) -> Result<()> {

    // Start transaction
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let caller_role = db::global_role(&mut tx, caller_id).await?;

    // Is the caller ranked high enough to change other's roles
    let permissible_roles = [GlobalRole::Admin];
    if !permissible_roles.contains(&caller_role) {
        return Err(AppError::Forbidden);
    }

    // Admins can't change each other's roles
    let target_role = db::global_role(&mut tx, target_id).await?;
    if caller_role == GlobalRole::Admin && target_role == GlobalRole::Admin {
        return Err(AppError::Forbidden);
    }

    // No row updated, so the target does not exist
    if !db::set_global_role(&mut tx, target_id, role).await? {
        return Err(AppError::NotFound);
    }

    tx.commit().await?;

    Ok(())
}

// Helper Methods //

/// Deletes every message an account wrote.
///
/// # Arguments
///
/// * `conn` - Connection to SQL DB.
/// * `user_id` - Author whose messages are deleted.
async fn delete_messages(
    conn: &mut sqlx::SqliteConnection,
    user_id: Uuid,
) -> Result<()> {

    // Attachments cascade
    sqlx::query("DELETE FROM messages WHERE author_id = ?1")
    .bind(user_id)
    .execute(&mut *conn)
    .await?;

    Ok(())
}

/// Removes an account from every room it belongs to, one room at a time so each
/// runs the same cull and promotion checks a single removal does, returning the
/// rooms it was in so the caller can tell each one.
///
/// # Arguments
///
/// * `conn` - Connection to SQL DB.
/// * `user_id` - Account being removed.
async fn leave_every_room(
    conn: &mut sqlx::SqliteConnection,
    user_id: Uuid,
) -> Result<Vec<Uuid>> {

    let rooms = db::member_room_ids(&mut *conn, user_id).await?;

    for room_id in &rooms {
        access::remove(&mut *conn, *room_id, user_id).await?;
    }

    Ok(rooms)
}
