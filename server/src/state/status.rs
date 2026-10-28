//! What is sent to a viewer about a user, and who it is sent to.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use uuid::Uuid;

use crate::sockets::session::events::ServerEvent;
use crate::sockets::session::registry;
use crate::state::platforms;
use crate::state;
use crate::state::AppState;
use crate::types::platforms::Platform;
use crate::types::status::{Appearance, UserState};
use crate::utils;

/// The status each user last declared, keyed by user id
#[derive(Clone, Default)]
pub struct Statuses {
    by_user: Arc<RwLock<HashMap<Uuid, Appearance>>>
}

/// Reads the status a user last declared
pub fn get(
    state: &AppState,
    user_id: Uuid,
) -> Option<Appearance> {
    let held = utils::read_lock(&state.statuses.by_user);

    match held.get(&user_id) {
        Some(appearance) => Some(*appearance),
        None => None
    }
}

// Write the status a user declares
pub fn set(
    state: &AppState,
    user_id: Uuid,
    appearance: Appearance,
) {
    let mut held = utils::write_lock(&state.statuses.by_user);
    held.insert(user_id, appearance);
}

// Build the status of every visible member
pub fn snapshot(
    state: &AppState,
    members: &[Uuid],
) -> Vec<UserState> {
    let mut users = Vec::new();
    for member_id in members {
        let member_id = *member_id;

        // A member holding no connection is Offline
        if !registry::is_connected(&state.registry, member_id) {
            continue;
        }

        let Some(appearance) = get(state, member_id) else {
            continue;
        };

        // An Offline member is left out, the same as one holding no connection
        if appearance == Appearance::Offline {
            continue;
        }

        users.push(UserState {
            user_id: member_id,
            appearance,
            device: registry::device(&state.registry, member_id)
        });
    }

    users
}

// Apply the status a new socket connects with, and announce the result
pub async fn connected(
    state: &AppState,
    user_id: Uuid,
    status: Appearance,
    already_connected: bool,
) {
    let mut current_status = None;
    if already_connected {
        current_status = get(state, user_id);
    }

    match current_status {
        None => {
            set(state, user_id, status);
        },
        Some(Appearance::Online) => {},
        Some(Appearance::Busy) => {},
        Some(Appearance::Away) => {
            if status == Appearance::Online {
                set(state, user_id, status);
            }
        },
        Some(Appearance::Offline) => {
            if status == Appearance::Online || status == Appearance::Away {
                set(state, user_id, status);
            }
        }
    }

    if let Some(current) = get(state, user_id) {
        announce(state, user_id, current).await;
    }
}

// Announce a user as Offline once their last socket closes
pub async fn disconnected(
    state: &AppState,
    user_id: Uuid,
) {
    if !registry::is_connected(&state.registry, user_id) {
        announce(state, user_id, Appearance::Offline).await;
    }
}

/// Sends the new member's status to the room
/// and a snapshot of everyone visible to the new member.
pub async fn on_join(
    state: &AppState,
    user_id: Uuid,
) {
    // Nothing to send for a user holding no connection
    if !registry::is_connected(&state.registry, user_id) {
        return;
    }

    let Some(appearance) = get(state, user_id) else {
        return;
    };

    announce(state, user_id, appearance).await;

    // Send all snapshots
    for event in state::snapshots(state, user_id).await {
        registry::inform_user(&state.registry, user_id, event);
    }
}

/// Sends a user's status to everyone sharing a room with them
pub async fn announce(
    state: &AppState,
    user_id: Uuid,
    status: Appearance,
) {
    // Offline users have no platform presence
    if status == Appearance::Offline {
        platforms::clear_metadata(state, user_id, Platform::Xbox).await;
        platforms::clear_metadata(state, user_id, Platform::Steam).await;
    }

    // Most recent device
    let device = registry::device(&state.registry, user_id);

    let event = ServerEvent::StatusUpdated { user_id, status, device };
    state::inform_shared_members(state, user_id, event).await;
}
