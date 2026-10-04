//! HTTP handlers for users.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::api::linked_accounts;
use crate::api::users::UserSummary;
use crate::db;
use crate::error::{AppError, Result};
use crate::routes::AuthUser;
use crate::shared::{GlobalRole, LinkedAccount};
use crate::sockets::events::ServerEvent;
use crate::sockets::{registry};
use crate::state::AppState;
use crate::validate;
use crate::{api, config};

// Data Structs //


/// A user's profile
#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/users.ts"))]
pub struct UserProfileResponse {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub description: String,
    pub avatar_file_id: Option<Uuid>,
    pub banner_file_id: Option<Uuid>,
    pub global_role: GlobalRole,
    pub created_at: i64,
    pub deleted_at: Option<i64>,
    pub links: Vec<LinkedAccount>,
}

/// What a member row or search result shows of a user
#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/users.ts"))]
pub struct UserSummaryResponse {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_file_id: Option<Uuid>,
    pub banner_file_id: Option<Uuid>,
}

impl From<UserSummary> for UserSummaryResponse {
    fn from(user: UserSummary) -> Self {
        Self {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
            avatar_file_id: user.avatar_file_id,
            banner_file_id: user.banner_file_id,
        }
    }
}

/// PATCH body for a user's own profile. An absent field is left as it stands.
#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/users.ts"))]
pub struct ProfilePatch {
    #[serde(default)]
    pub display_name: Option<String>,

    /// Empty string clears the text
    #[serde(default)]
    pub description: Option<String>,

    /// Nil UUID clears the avatar
    #[serde(default)]
    pub avatar_file_id: Option<Uuid>,

    /// Nil UUID clears the banner
    #[serde(default)]
    pub banner_file_id: Option<Uuid>,
}

/// PATCH body for changing a user's global role.
#[derive(Deserialize)]
pub struct SetRoleRequest {
    pub role: GlobalRole,
}

/// PATCH body for replacing a password.
#[derive(Deserialize)]
pub struct PasswordRequest {
    pub current_password: String,
    pub new_password: String,

    /// Revokes every session but the one making the request
    #[serde(default)]
    pub revoke_others: bool,
}

/// PUT body for handing the server to another account.
#[derive(Deserialize)]
pub struct TransferOwnershipRequest {
    pub user_id: Uuid,

    /// Role the outgoing Owner keeps
    pub demote_to: GlobalRole,
}

/// DELETE body for closing your own account.
#[derive(Deserialize)]
pub struct DeleteAccountRequest {
    /// Replaces the names and releases the username
    #[serde(default)]
    pub anonymize: bool,

    /// Tombstones every message the account wrote
    #[serde(default)]
    pub delete_history: bool,
}

/// The caller's own preferences
#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/users.ts"))]
pub struct PreferencesResponse {
    pub room_layout: String,
}

/// PATCH body for the caller's own preferences. An absent field is left as it stands
#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/users.ts"))]
pub struct PreferencesPatch {
    #[serde(default)]
    pub room_layout: Option<String>,
}

/// POST body for looking up users.
#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/users.ts", optional_fields))]
pub struct UsersLookup {
    #[serde(default)]
    pub ids: Option<Vec<Uuid>>,

    /// Start of a username to match
    #[serde(default)]
    pub username: Option<String>,

    #[serde(default)]
    pub after: Option<Uuid>,

    #[serde(default)]
    pub limit: Option<i64>,
}

// Routing Methods //

/// Gets the summaries of a set of users.
///
/// # Arguments
///
/// * `pool` - Pool of SQL connections.
/// * `body` - Ids to fetch, or a username prefix to page through.
pub async fn get_users(
    AuthUser(..): AuthUser,
    State(pool): State<SqlitePool>,
    Json(body): Json<UsersLookup>,
) -> Result<Json<Vec<UserSummaryResponse>>> {

    let users = match body.ids {

        // Look up users by id
        Some(ids) => {
            // Reject ids mixed with the search fields
            if body.username.is_some() || body.after.is_some() || body.limit.is_some() {
                return Err(AppError::Validation(
                    "ids cannot be combined with username, after, or limit".to_string()
                ));
            }

            // Reject more ids than the lookup limit
            let max = config::get().limits.users_lookup;
            if ids.len() > max {
                return Err(AppError::Validation(format!("at most {max} ids per lookup")));
            }

            api::users::by_ids(&pool, &ids).await?
        }

        // Search users by username prefix, one page at a time
        None => {
            let max = config::get().limits.users_page;
            let limit = body.limit.unwrap_or(max).clamp(1, max);

            api::users::by_username_prefix(&pool, body.username, body.after, limit).await?
        }
    };

    Ok(Json(users.into_iter().map(UserSummaryResponse::from).collect()))
}

/// Gets a user's public profile.
///
/// # Arguments
///
/// * `pool` - Pool of SQL connections.
/// * `user_id` - User to look up.
pub async fn get_user(
    AuthUser(..): AuthUser,
    State(pool): State<SqlitePool>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserProfileResponse>> {
    Ok(Json(build_profile(&pool, user_id).await?))
}

/// Gets the caller's own profile.
///
/// # Arguments
///
/// * `user_id` - Whose profile to return.
/// * `pool` - Pool of SQL connections.
pub async fn get_me(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
) -> Result<Json<UserProfileResponse>> {
    Ok(Json(build_profile(&pool, user_id).await?))
}

/// Build a public profile based on user related components
///
/// Returns `AppError::NotFound` if no such user exists.
async fn build_profile(pool: &SqlitePool, user_id: Uuid) -> Result<UserProfileResponse> {
    let row = api::users::get(pool, user_id).await?;
    let links = linked_accounts::list(pool, user_id).await?;

    Ok(UserProfileResponse {
        id: row.id,
        username: row.username,
        display_name: row.display_name,
        description: row.description,
        avatar_file_id: row.avatar_file_id,
        banner_file_id: row.banner_file_id,
        global_role: row.global_role,
        created_at: row.created_at,
        deleted_at: row.deleted_at,
        links,
    })
}

/// Writes the caller's own profile.
///
/// # Arguments
///
/// * `user_id` - Whose profile is being written.
/// * `app_state` - Pool and socket registry.
/// * `body` - Fields to change.
pub async fn update_me(
    AuthUser(user_id, ..): AuthUser,
    State(app_state): State<AppState>,
    Json(body): Json<ProfilePatch>,
) -> Result<StatusCode> {
    // Profile as stored, or None if no such user
    let updated = api::users::update(
        &app_state.pool,
        user_id,
        body.display_name,
        body.description,
        body.avatar_file_id,
        body.banner_file_id,
    ).await?;

    // Notify everyone sharing a room of the new name and pictures
    if let Some(profile) = updated {
        let mut conn = app_state.pool.acquire().await?;
        let members = db::shared_room_member_ids(&mut conn, user_id).await?;

        let event = ServerEvent::ProfileUpdated {
            user_id,
            display_name: profile.display_name,
            description: profile.description,
            avatar_file_id: profile.avatar_file_id,
            banner_file_id: profile.banner_file_id,
        };
        registry::inform_users(&app_state, &members, event);
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Gets the caller's own preferences
pub async fn get_preferences(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
) -> Result<Json<PreferencesResponse>> {

    let mut conn = pool.acquire().await?;
    let room_layout = db::get_preferences(&mut conn, user_id).await?;

    Ok(Json(PreferencesResponse { room_layout }))
}

/// Writes the caller's own preferences
pub async fn update_preferences(
    AuthUser(user_id, ..): AuthUser,
    State(app_state): State<AppState>,
    Json(body): Json<PreferencesPatch>,
) -> Result<StatusCode> {

    // Saves the caller's room list layout
    if let Some(room_layout) = body.room_layout {
        validate::room_layout(&room_layout)?;

        let mut conn = app_state.pool.acquire().await?;
        db::set_room_layout(&mut conn, user_id, &room_layout).await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Replaces the caller's password.
///
/// # Arguments
///
/// * `user_id` - Whose password is being replaced.
/// * `session_hash` - The caller's own session, kept when revoking the rest.
/// * `pool` - Pool of SQL connections.
/// * `body` - Current password, replacement, and whether to revoke elsewhere.
pub async fn update_my_password(
    AuthUser(user_id, session_hash): AuthUser,
    State(pool): State<SqlitePool>,
    Json(body): Json<PasswordRequest>,
) -> Result<StatusCode> {
    api::auth::change_password(
        &pool,
        user_id,
        &body.current_password,
        &body.new_password,
        body.revoke_others,
        &session_hash,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Hands the server to another account.
///
/// # Arguments
///
/// * `caller_id` - The Owner giving the server away.
/// * `pool` - Pool of SQL connections.
/// * `body` - Account receiving Owner, and the role the caller keeps.
pub async fn transfer_ownership(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Json(body): Json<TransferOwnershipRequest>,
) -> Result<StatusCode> {
    api::users::transfer_ownership(&pool, caller_id, body.user_id, body.demote_to).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Closes the caller's own account.
///
/// # Arguments
///
/// * `user_id` - Account being closed.
/// * `app_state` - Pool and socket registry.
/// * `body` - Whether to anonymize the names and whether to drop the history.
pub async fn delete_me(
    AuthUser(user_id, ..): AuthUser,
    State(app_state): State<AppState>,
    Json(body): Json<DeleteAccountRequest>,
) -> Result<StatusCode> {
    let rooms = api::users::delete(
        &app_state.pool,
        user_id,
        body.anonymize,
        body.delete_history,
    )
    .await?;

    broadcast_member_left(&app_state, user_id, rooms).await;

    Ok(StatusCode::NO_CONTENT)
}

/// Closes someone else's account.
///
/// # Arguments
///
/// * `caller_id` - Who is closing the account.
/// * `app_state` - Pool and socket registry.
/// * `target_id` - Account being closed.
/// * `body` - Whether to anonymize the names.
pub async fn delete_user(
    AuthUser(caller_id, ..): AuthUser,
    State(app_state): State<AppState>,
    Path(target_id): Path<Uuid>,
    Json(body): Json<DeleteAccountRequest>,
) -> Result<StatusCode> {
    let rooms =
        api::users::delete_other(&app_state.pool, caller_id, target_id, body.anonymize).await?;

    broadcast_member_left(&app_state, target_id, rooms).await;

    Ok(StatusCode::NO_CONTENT)
}

/// Tells each of several rooms that a member is no longer in it.
///
/// # Arguments
///
/// * `app_state` - Pool and socket registry.
/// * `user_id` - Member that was removed.
/// * `rooms` - Rooms the membership was removed from.
async fn broadcast_member_left(app_state: &AppState, user_id: Uuid, rooms: Vec<Uuid>) {
    for room_id in rooms {
        let event = ServerEvent::MemberLeft { room_id, user_id };
        registry::broadcast(app_state, room_id, event).await;
    }
}

/// Promotes or demotes a user.
///
/// # Arguments
///
/// * `caller_id` - Who is making the change.
/// * `pool` - Pool of SQL connections.
/// * `target_id` - User whose role changes.
/// * `body` - The role to set.
pub async fn set_role(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Path(target_id): Path<Uuid>,
    Json(body): Json<SetRoleRequest>,
) -> Result<StatusCode> {
    api::users::set_role(&pool, caller_id, target_id, body.role).await?;

    Ok(StatusCode::NO_CONTENT)
}
