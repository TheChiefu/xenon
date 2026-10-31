//! The appearance each user declares, what a viewer is sent about it, and who it is sent to.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use uuid::Uuid;

use crate::sockets::session::events::ServerEvent;
use crate::sockets::session::registry;
use crate::state::{self, platforms, AppState};
use crate::types::appearance::{Appearance, UserState};
use crate::utils;

/// The appearance each user last declared, keyed by user id
#[derive(Clone, Default)]
pub struct Appearances {
    by_user: Arc<RwLock<HashMap<Uuid, Appearance>>>
}

/// Reads the appearance a user last declared
pub fn get(
    appearances: &Appearances,
    user_id: Uuid,
) -> Option<Appearance> {
    let held = utils::read_lock(&appearances.by_user);

    match held.get(&user_id) {
        Some(appearance) => Some(*appearance),
        None => None
    }
}

/// Write the appearance a user declares
pub fn set(
    appearances: &Appearances,
    user_id: Uuid,
    appearance: Appearance,
) {
    let mut held = utils::write_lock(&appearances.by_user);
    held.insert(user_id, appearance);
}

/// Build the appearance of every visible member
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

        let Some(appearance) = get(&state.appearances, member_id) else {
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

/// Apply the appearance a new socket connects with, and announce the result
pub async fn on_connect(
    state: &AppState,
    user_id: Uuid,
    appearance: Appearance,
    already_connected: bool,
) {
    let mut current_appearance = None;
    if already_connected {
        current_appearance = get(&state.appearances, user_id);
    }

    match current_appearance {
        None => {
            set(&state.appearances, user_id, appearance);
        },
        Some(Appearance::Online) => {},
        Some(Appearance::Busy) => {},
        Some(Appearance::Away) => {
            if appearance == Appearance::Online {
                set(&state.appearances, user_id, appearance);
            }
        },
        Some(Appearance::Offline) => {
            if appearance == Appearance::Online || appearance == Appearance::Away {
                set(&state.appearances, user_id, appearance);
            }
        }
    }

    if let Some(current) = get(&state.appearances, user_id) {
        if current == Appearance::Offline {
            platforms::clear_all(state, user_id).await;
        }
        announce(state, user_id, current).await;
    }
}

/// Announce a user as Offline once their last socket closes
pub async fn on_disconnect(
    state: &AppState,
    user_id: Uuid,
) {
    if !registry::is_connected(&state.registry, user_id) {
        announce(state, user_id, Appearance::Offline).await;
    }
}

/// Sends the new member's appearance to everyone sharing a room with them,
/// and sends the new member a snapshot of everyone visible to them.
pub async fn on_join(
    state: &AppState,
    user_id: Uuid,
) {
    // Nothing to send for a user holding no connection
    if !registry::is_connected(&state.registry, user_id) {
        return;
    }

    let Some(appearance) = get(&state.appearances, user_id) else {
        return;
    };

    announce(state, user_id, appearance).await;

    for event in state::snapshots(state, user_id).await {
        registry::inform_users(&state.registry, &[user_id], event);
    }
}

/// Sends a user's appearance to everyone sharing a room with them
pub async fn announce(
    state: &AppState,
    user_id: Uuid,
    appearance: Appearance,
) {
    // Most recent device
    let device = registry::device(&state.registry, user_id);

    let event = ServerEvent::AppearanceUpdated { user_id, appearance, device };
    state::inform_shared_members(state, user_id, event).await;
}
