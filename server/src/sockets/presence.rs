//! What is sent to a viewer about a user, and who it is sent to.

use uuid::Uuid;

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::db;
use crate::error::Result;
use crate::shared::Status;
use crate::sockets::events::{ServerEvent, UserPresence};
use crate::sockets::game_presence;
use crate::sockets::registry;
use crate::state::AppState;

/// What a user is connected from, for a client to show beside their status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "sockets/events.ts", repr(enum = name)))]
#[serde(rename_all = "lowercase")]
pub enum Device {
    Windows,
    Macos,
    Linux,
    Android,
    Ios,
    Chrome,
    Desktop,
    Mobile,
    Tablet
}

/// Builds the presence list a connecting user is sent. It contains everyone
/// they share a room with, apart from anyone reported as Offline.
pub async fn snapshot(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<UserPresence>> {

    // The members whose presence this user may see
    let mut conn = state.pool.acquire().await?;
    let members = db::shared_room_member_ids(&mut conn, user_id).await?;

    let mut users = Vec::new();
    for declared in registry::statuses_of(state, &members) {

        // An Offline member is left out, the same as one holding no connection
        if declared.status == Status::Offline {
            continue;
        }

        users.push(UserPresence {
            user_id: declared.user_id,
            status: declared.status,
            device: declared.device
        });
    }

    Ok(users)
}

/// Sends the new member's presence to the room
/// and a snapshot of everyone visible to the new member.
pub async fn on_join(
    state: &AppState,
    user_id: Uuid,
) {
    // Nothing to send for a user holding no connection
    let Some(declared) = registry::status_of(state, user_id) else {
        return;
    };

    // No previous status: the room could not see the new member until now
    on_change(state, user_id, None, Some(declared.status)).await;

    // A failed snapshot is logged, and the member joins with no presence list
    match snapshot(state, user_id).await {
        Ok(users) => {
            let event = ServerEvent::PresenceSnapshot { users };
            registry::inform_user(state, user_id, event);
        }
        Err(e) => tracing::error!("could not build a presence snapshot for {user_id}: {e}")
    }
}

/// Sends a user's new presence to everyone sharing a room with them, when it
/// differs from the presence they had before.
///
/// # Arguments
///
/// * `before` - Status they held, or `None` before they connected.
/// * `after` - Status they hold now, or `None` once disconnected.
pub async fn on_change(
    state: &AppState,
    user_id: Uuid,
    before: Option<Status>,
    after: Option<Status>,
) {
    // Offline users have no game presence
    if after == Some(Status::Offline) && before != Some(Status::Offline) {
        game_presence::clear(state, user_id).await;
    }

    // An Offline user connecting or leaving is not a change to a viewer
    let status = after.unwrap_or(Status::Offline);
    if status == before.unwrap_or(Status::Offline) {
        return;
    }

    // A user with no connection has no device
    let device = match registry::status_of(state, user_id) {
        Some(entry) => entry.device,
        None => None
    };

    let event = ServerEvent::PresenceUpdated { user_id, status, device };
    registry::inform_shared_members(state, user_id, event).await;
}
