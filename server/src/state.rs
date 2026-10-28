//! What every handler is given:
//! - the database
//! - who is connected

pub mod platforms;
pub mod status;

use axum::extract::FromRef;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db;
use crate::error::Result;
use crate::ratelimit::{AddressLimits, LoginFailures};
use crate::sockets::call;
use crate::sockets::call::registry::ActiveCalls;
use crate::sockets::session::events::ServerEvent;
use crate::sockets::session::registry::{self, Registry};
use crate::state::platforms::ReportedPlatforms;
use crate::state::status::Statuses;
use crate::types::status::Appearance;

/// Cloned per request
///
/// `FromRef` generates a per-field accessor
/// letting handlers keep asking for `State<SqlitePool>`
#[derive(Clone, FromRef)]
pub struct AppState {
    pub pool: SqlitePool,
    pub registry: Registry,

    /// The status each user last declared
    pub statuses: Statuses,

    /// Who is in each room's call
    pub calls: ActiveCalls,

    /// What each user's linked account last reported
    pub platforms: ReportedPlatforms,

    /// Failures each IP has made, and whether it is blocked
    pub address_limits: AddressLimits,

    /// Failed logins each account has made since its last successful login
    pub login_failures: LoginFailures
}

impl AppState {

    /// Builds the state a router is handed.
    ///
    /// # Arguments
    ///
    /// * `pool` - Pool of SQL connections.
    pub fn new(pool: SqlitePool) -> Self {
        AppState {
            pool,
            registry: Registry::default(),
            statuses: Statuses::default(),
            calls: ActiveCalls::default(),
            platforms: ReportedPlatforms::default(),
            address_limits: AddressLimits::default(),
            login_failures: LoginFailures::default(),
        }
    }
}

// Close every websocket a user has
pub async fn disconnect(
    state: &AppState,
    user_id: Uuid,
) {
    // Sign the user's clients out before their sockets close
    registry::inform_user(&state.registry, user_id, ServerEvent::SessionRevoked);
    registry::remove(&state.registry, user_id);

    // Tell others the user is offline
    status::announce(state, user_id, Appearance::Offline).await;

    leave_call(state, user_id).await;
}

// Remove a user from their call and tell its room
pub async fn leave_call(
    state: &AppState,
    user_id: Uuid,
) {
    if let Some(room_id) = call::registry::leave(&state.calls, user_id) {
        let event = ServerEvent::ParticipantLeft { room_id, user_id };
        broadcast(state, room_id, event).await;
    }
}

// Build every snapshot a viewer is sent
pub async fn snapshots(
    state: &AppState,
    viewer_id: Uuid,
) -> Vec<ServerEvent> {
    let mut events = Vec::new();

    match shared_members(state, viewer_id).await {
        Ok(members) => {
            events.push(
                ServerEvent::StatusSnapshot {
                    users: status::snapshot(state, &members)
                }
            );
            events.push(
                ServerEvent::PlatformSnapshot {
                    users: platforms::snapshot(state, &members)
                }
            );
        }
        Err(e) => tracing::error!("could not read who shares a room with {viewer_id}: {e}")
    }

    match member_rooms(state, viewer_id).await {
        Ok(room_ids) => {
            events.push(
                ServerEvent::CallSnapshot {
                    calls: call::registry::snapshot(&state.calls, &room_ids)
                }
            );
        }
        Err(e) => tracing::error!("could not read the rooms of {viewer_id}: {e}")
    }

    events
}

// Send an event to every member of a room
pub async fn broadcast(
    state: &AppState,
    room_id: Uuid,
    event: ServerEvent,
) {
    let mut conn = match state.pool.acquire().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("broadcast could not acquire a connection: {e}");
            return;
        }
    };

    let members = match db::room_member_ids(&mut conn, room_id).await {
        Ok(members) => members,
        Err(e) => {
            tracing::error!("could not acquire room members: {e}");
            return;
        }
    };

    registry::inform_users(&state.registry, &members, event);
}

// Notify several rooms that a member is no longer in it
pub async fn broadcast_member_left(
    state: &AppState,
    user_id: Uuid,
    rooms: Vec<Uuid>,
) {
    for room_id in rooms {
        let event = ServerEvent::MemberLeft { room_id, user_id };
        broadcast(state, room_id, event).await;
    }
}

// Send an event to everyone who shares a room with a user
pub async fn inform_shared_members(
    state: &AppState,
    user_id: Uuid,
    event: ServerEvent,
) {
    let members = match shared_members(state, user_id).await {
        Ok(members) => members,
        Err(e) => {
            tracing::error!("could not read who shares a room with {user_id}: {e}");
            return;
        }
    };

    registry::inform_users(&state.registry, &members, event);
}

// Read everyone who shares a room with a user
pub async fn shared_members(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<Uuid>> {
    let mut conn = state.pool.acquire().await?;
    let members = db::shared_room_member_ids(&mut conn, user_id).await?;
    Ok(members)
}

// Read every room a user is a member of
pub async fn member_rooms(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<Uuid>> {
    let mut conn = state.pool.acquire().await?;
    let room_ids = db::member_room_ids(&mut conn, user_id).await?;
    Ok(room_ids)
}
