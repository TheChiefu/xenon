//! HTTP handlers for rooms and the tables scoped to a single room

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::error::{AppError, Result};
use crate::routes::AuthUser;
use crate::sockets::call;
use crate::sockets::session::events::ServerEvent;
use crate::sockets::session::registry;
use crate::state::status;
use crate::state::{self, AppState};
use crate::types::rooms::{Notify, Permission, RoomInfo, Visibility};
use crate::api;
use crate::config;

// REQUESTS //

#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/rooms.ts"))]
pub struct RequestCreateRoom {
    pub name: String,
    pub visibility: Visibility,
    pub default_permissions: Vec<Permission>,
    pub claim_all: bool,
}

#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/rooms.ts"))]
pub struct RequestRead {
    pub last_read_seq: i64,
}

// RESPONSES //

#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/rooms.ts"))]
pub struct RespondCreated {
    pub id: Uuid,
}

#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/rooms.ts"))]
pub struct RespondDiscoverability {
    pub id: Uuid,
    pub name: String,
    pub visibility: Visibility,
    pub created_at: i64,
}

#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/rooms.ts"))]
pub struct RespondMember {
    pub user_id: Uuid,
    pub permissions: Vec<Permission>,
    pub granted_at: i64,
    pub notify: Option<Notify>
}

#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/rooms.ts"))]
pub struct RespondAccess {
    pub permissions: Option<Vec<Permission>>,
    pub invited: bool,
    pub banned: bool,
}

// PATCHES //

#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/rooms.ts"))]
pub struct Patch {
    pub name: Option<String>,
    pub visibility: Option<Visibility>,
    pub default_permissions: Option<Vec<Permission>>,
}

#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/rooms.ts"))]
pub struct PatchAccess {
    pub permissions: Option<Vec<Permission>>,
    pub notify: Option<Notify>
}

// DATA STRUCTS //

/// Query string for a paged room listing
#[derive(Deserialize)]
pub struct DirectoryQuery {
    pub after: Option<Uuid>,
    pub limit: Option<i64>
}

// ROUTING METHODS //

/// List the members of a room
pub async fn list_members(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Path(room_id): Path<Uuid>,
) -> Result<Json<Vec<RespondMember>>> {

    let members = api::rooms::access::list(&pool, room_id, caller_id).await?;

    let mut response = Vec::with_capacity(members.len());
    for member in members {
        response.push(RespondMember {
            user_id: member.user_id,
            permissions: member.permissions.0,
            granted_at: member.granted_at,
            notify: member.notify
        });
    }

    Ok(Json(response))
}

/// Get a user's access to a room
pub async fn get_member(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Path((room_id, target_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<RespondAccess>> {

    let access = api::rooms::access::get(&pool, room_id, caller_id, target_id).await?;

    Ok(Json(RespondAccess {
        permissions: access.permissions,
        invited: access.invited,
        banned: access.banned
    }))
}

/// Patch a member's information in a room
pub async fn patch_user(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Path((room_id, target_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<PatchAccess>,
) -> Result<StatusCode> {

    // Notify can only be set on the caller's own membership
    if body.notify.is_some() && caller_id != target_id {
        return Err(AppError::Forbidden);
    }

    if let Some(permissions) = body.permissions {
        api::rooms::access::update(&pool, room_id, caller_id, target_id, permissions).await?;
    }

    // Set caller's notification preference
    if let Some(notify) = body.notify {
        api::rooms::access::set_notify(&pool, room_id, caller_id, notify).await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Invite a user to a room
pub async fn invite_user(
    AuthUser(caller_id, ..): AuthUser,
    State(app_state): State<AppState>,
    Path((room_id, target_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {

    api::rooms::invites::create(&app_state.pool, room_id, caller_id, target_id).await?;

    let event = ServerEvent::Invited { room_id };
    registry::inform_users(&app_state.registry, &[target_id], event);

    Ok(StatusCode::NO_CONTENT)
}

/// Get the users a room has invited
pub async fn list_invites(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Path(room_id): Path<Uuid>,
) -> Result<Json<Vec<Uuid>>> {

    let user_ids = api::rooms::invites::list(&pool, room_id, caller_id).await?;

    Ok(Json(user_ids))
}

/// Withdraw an invite the room issued
pub async fn revoke_invite(
    AuthUser(caller_id, ..): AuthUser,
    State(app_state): State<AppState>,
    Path((room_id, target_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {

    api::rooms::invites::revoke(&app_state.pool, room_id, caller_id, target_id).await?;

    // Notify user invite was revoked
    let event = ServerEvent::InviteRevoked { room_id };
    registry::inform_users(&app_state.registry, &[target_id], event);

    Ok(StatusCode::NO_CONTENT)
}

/// Get list of users banned from a room
pub async fn list_bans(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Path(room_id): Path<Uuid>,
) -> Result<Json<Vec<Uuid>>> {

    let user_ids = api::rooms::bans::list(&pool, room_id, caller_id).await?;

    Ok(Json(user_ids))
}

/// Ban a user from the room
pub async fn ban_user(
    AuthUser(caller_id, ..): AuthUser,
    State(app_state): State<AppState>,
    Path((room_id, target_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {

    api::rooms::bans::create(&app_state.pool, room_id, caller_id, target_id).await?;

    let event = ServerEvent::Banned { room_id };
    registry::inform_users(&app_state.registry, &[target_id], event);

    // Ban removes the membership, notify members user left
    let event = ServerEvent::MemberLeft { room_id, user_id: target_id };
    state::broadcast(&app_state, room_id, event).await;

    // Ban removes the user from the room's call
    if call::registry::participant_call(&app_state.calls, target_id) == Some(room_id) {
        state::leave_call(&app_state, target_id).await;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Unban a user from a room
pub async fn unban_user(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Path((room_id, target_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {

    api::rooms::bans::revoke(&pool, room_id, caller_id, target_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Update a room
pub async fn update(
    AuthUser(caller_id, ..): AuthUser,
    State(app_state): State<AppState>,
    Path(room_id): Path<Uuid>,
    Json(body): Json<Patch>,
) -> Result<StatusCode> {

    // Update the room with the given patch
    api::rooms::update(
        &app_state.pool,
        room_id,
        caller_id,
        body.name.as_deref(),
        body.visibility,
        body.default_permissions
    ).await?;

    // Notify all room members, room has updated
    let event = ServerEvent::RoomUpdated { room_id };
    state::broadcast(&app_state, room_id, event).await;

    Ok(StatusCode::NO_CONTENT)
}

/// Create a room
pub async fn create(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Json(body): Json<RequestCreateRoom>,
) -> Result<(StatusCode, Json<RespondCreated>)> {

    let default_permissions = body.default_permissions;

    // The creator takes every permission, or the template everyone else joins on
    let caller_permissions = if body.claim_all {
        Permission::ALL.to_vec()
    } else {
        default_permissions.clone()
    };

    // Attempt to create room
    let id = api::rooms::create(
        &pool,
        caller_id,
        &body.name,
        caller_permissions,
        default_permissions,
        body.visibility
    ).await?;

    Ok((StatusCode::CREATED, Json(RespondCreated { id })))
}

/// Delete a room
pub async fn delete(
    AuthUser(caller_id, ..): AuthUser,
    State(app_state): State<AppState>,
    Path(room_id): Path<Uuid>,
) -> Result<StatusCode> {

    let members = api::rooms::delete(&app_state.pool, room_id, caller_id).await?;
    call::registry::end(&app_state.calls, room_id);

    // Notify everyone who was in the room
    let event = ServerEvent::RoomDeleted { room_id };
    registry::inform_users(&app_state.registry, &members, event);

    Ok(StatusCode::NO_CONTENT)
}

/// Join a room
pub async fn join(
    AuthUser(user_id, ..): AuthUser,
    State(app_state): State<AppState>,
    Path(room_id): Path<Uuid>,
) -> Result<StatusCode> {

    api::rooms::join(&app_state.pool, room_id, user_id).await?;

    let event = ServerEvent::MemberJoined { room_id, user_id };
    state::broadcast(&app_state, room_id, event).await;

    status::on_join(&app_state, user_id).await;

    Ok(StatusCode::NO_CONTENT)
}

/// Leave a room
pub async fn leave(
    AuthUser(user_id, ..): AuthUser,
    State(app_state): State<AppState>,
    Path(room_id): Path<Uuid>,
) -> Result<StatusCode> {

    api::rooms::leave(&app_state.pool, room_id, user_id).await?;

    // The membership is already gone, so this reaches the remaining members
    let event = ServerEvent::MemberLeft { room_id, user_id };
    state::broadcast(&app_state, room_id, event.clone()).await;
    registry::inform_users(&app_state.registry, &[user_id], event);

    Ok(StatusCode::NO_CONTENT)
}

/// Store the caller's read position in a room
pub async fn mark_read(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Path(room_id): Path<Uuid>,
    Json(body): Json<RequestRead>,
) -> Result<StatusCode> {

    api::rooms::read::mark(&pool, room_id, user_id, body.last_read_seq).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get one room
pub async fn get(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Path(room_id): Path<Uuid>,
) -> Result<Json<RoomInfo>> {

    let room = api::rooms::get(&pool, room_id, caller_id).await?;

    Ok(Json(RoomInfo::from(room)))
}

/// Get one page of the Public rooms on the server
pub async fn list_discoverable(
    AuthUser(..): AuthUser,
    State(pool): State<SqlitePool>,
    Query(query): Query<DirectoryQuery>,
) -> Result<Json<Vec<RespondDiscoverability>>> {

    let max = config::get().responses.rooms_per_page;
    let limit = query.limit.unwrap_or(max).clamp(1, max);

    let rooms = api::rooms::list_discoverable(&pool, query.after, limit).await?;

    let mut response = Vec::with_capacity(rooms.len());
    for room in rooms {
        response.push(RespondDiscoverability {
            id: room.id,
            name: room.name,
            visibility: room.visibility,
            created_at: room.created_at
        });
    }

    Ok(Json(response))
}
