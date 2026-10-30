//! HTTP handlers for the caller's own account.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::api;
use crate::error::Result;
use crate::routes::AuthUser;
use crate::sockets::session::events::ServerEvent;
use crate::state::{self, AppState};
use crate::types::rooms::RoomInfo;
use crate::types::users::UserProfile;

// REQUESTS //

#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/me.ts"))]
pub struct RequestPasswordSet {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/me.ts"))]
pub struct RequestDeleteAccount {
    /// Deletes every message the account wrote
    #[serde(default)]
    pub delete_history: bool,
}

// RESPONSES //

#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/me.ts"))]
pub struct RespondRoomPreferences {
    pub room_layout: String,
}

#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/me.ts"))]
pub struct RespondRoomReadState {
    pub room_id: Uuid,
    pub last_read_seq: i64,
    pub unread: i64,
}

#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/me.ts"))]
pub struct RespondRoomInvite {
    pub room_id: Uuid,
    pub room_name: Option<String>
}

// PATCHES //

#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/me.ts"))]
pub struct PatchUserProfile {
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

#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/me.ts"))]
pub struct PatchRoomPreferences {
    #[serde(default)]
    pub room_layout: Option<String>,
}

// ROUTING METHODS //

/// Get the caller's profile
pub async fn get(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
) -> Result<Json<UserProfile>> {
    let profile = api::users::profile(&pool, user_id).await?;
    Ok(Json(profile))
}

/// Update the caller's profile
pub async fn update(
    AuthUser(user_id, ..): AuthUser,
    State(app_state): State<AppState>,
    Json(body): Json<PatchUserProfile>,
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
        let event = ServerEvent::ProfileUpdated {
            user_id,
            display_name: profile.display_name,
            description: profile.description,
            avatar_file_id: profile.avatar_file_id,
            banner_file_id: profile.banner_file_id,
        };
        state::inform_shared_members(&app_state, user_id, event).await;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Get the caller's preferences
pub async fn get_preferences(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
) -> Result<Json<RespondRoomPreferences>> {
    let room_layout = api::users::get_preferences(&pool, user_id).await?;
    Ok(Json(RespondRoomPreferences { room_layout }))
}

/// Write the caller's preferences
pub async fn update_preferences(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Json(body): Json<PatchRoomPreferences>,
) -> Result<StatusCode> {

    // Save the caller's room list layout
    if let Some(room_layout) = body.room_layout {
        api::users::set_room_layout(&pool, user_id, &room_layout).await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Replace the caller's password (SHOULD WE MAKE GENERALIZE USER PATCH INSTEAD?)
pub async fn update_password(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Json(body): Json<RequestPasswordSet>,
) -> Result<StatusCode> {
    api::auth::change_password(
        &pool,
        user_id,
        &body.current_password,
        &body.new_password,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Revoke session the request authenticated with
pub async fn sign_out(
    AuthUser(_, session_hash): AuthUser,
    State(pool): State<SqlitePool>,
) -> Result<StatusCode> {
    api::auth::revoke_session(&pool, &session_hash).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Close the caller's account
pub async fn delete(
    AuthUser(user_id, ..): AuthUser,
    State(app_state): State<AppState>,
    Json(body): Json<RequestDeleteAccount>,
) -> Result<StatusCode> {
    let rooms = api::users::delete(&app_state.pool, user_id, body.delete_history).await?;
    state::disconnect(&app_state, user_id).await;
    state::broadcast_member_left(&app_state, user_id, rooms).await;
    Ok(StatusCode::NO_CONTENT)
}

/// Get the rooms the caller is a member of
pub async fn list_rooms(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<RoomInfo>>> {
    let rooms = api::rooms::list_mine(&pool, user_id).await?;
    let mut response = Vec::with_capacity(rooms.len());
    for room in rooms {
        response.push(RoomInfo::from(room));
    }

    Ok(Json(response))
}

/// Get the caller's read position in each room they are in
pub async fn list_reads(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<RespondRoomReadState>>> {
    let entries = api::rooms::read::list(&pool, user_id).await?;
    let mut response = Vec::with_capacity(entries.len());
    for entry in entries {
        response.push(RespondRoomReadState {
            room_id: entry.room_id,
            last_read_seq: entry.last_read_seq,
            unread: entry.unread
        });
    }

    Ok(Json(response))
}

/// Get the room invites the caller is a recipient of
pub async fn list_invites(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<RespondRoomInvite>>> {
    let invites = api::rooms::invites::list_for_user(&pool, user_id).await?;
    let mut response = Vec::with_capacity(invites.len());
    for invite in invites {
        response.push(RespondRoomInvite {
            room_id: invite.room_id,
            room_name: invite.room_name
        });
    }

    Ok(Json(response))
}

/// Decline a room invite
pub async fn decline_invite(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Path(room_id): Path<Uuid>,
) -> Result<StatusCode> {
    api::rooms::invites::decline(&pool, room_id, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
