//! What each linked account last reported, and who it is sent to.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use uuid::Uuid;

use crate::db;
use crate::error::Result;
use crate::shared::{Platform, Status};
use crate::sockets::events::ServerEvent;
use crate::sockets::events::platforms::{Activity, Presence, UserActivity};
use crate::sockets::chat::registry;
use crate::state::AppState;
use crate::utils::{read_lock, write_lock};

/// What each user's linked accounts last reported, keyed by user id
#[derive(Clone, Default)]
pub struct ReportedGames {
    by_user: Arc<RwLock<HashMap<Uuid, Activity>>>
}

// Stores a reported platform presence and sends it to everyone sharing a room with its owner
pub async fn report_presence(
    state: &AppState,
    user_id: Uuid,
    platform: Platform,
    presence: Presence,
) {
    let declared = match registry::status_of(state, user_id) {
        Some(entry) => Some(entry.status),
        None => None
    };

    // Offline users show no platform presence
    if declared == Some(Status::Offline) {
        remove_presence(state, user_id, platform);
        return;
    }

    store_presence(state, user_id, platform, presence.clone());

    let event = ServerEvent::GamePresenceUpdated {
        user_id,
        platform,
        presence: Some(presence)
    };
    registry::inform_shared_members(state, user_id, event).await;
}

/// Builds the platform presence list a connecting user is sent. It contains
/// everyone they share a room with who is on a linked account.
pub async fn snapshot(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<UserActivity>> {

    // The members whose platform presence this user may see
    let mut conn = state.pool.acquire().await?;
    let members = db::shared_room_member_ids(&mut conn, user_id).await?;

    let held = read_lock(&state.game_presence.by_user);

    let mut users = Vec::new();
    for member_id in members {
        let Some(activity) = held.get(&member_id) else {
            continue;
        };

        users.push(UserActivity {
            user_id: member_id,
            activity: activity.clone()
        });
    }

    Ok(users)
}

// Clear individual platform presence
pub async fn clear_presence(state: &AppState, user_id: Uuid, platform: Platform) {
    if remove_presence(state, user_id, platform) {
        inform_cleared(state, user_id, platform).await;
    }
}

// Send event that input user's platform presence cleared
async fn inform_cleared(state: &AppState, user_id: Uuid, platform: Platform) {
    let event = ServerEvent::GamePresenceUpdated { user_id, platform, presence: None };
    registry::inform_shared_members(state, user_id, event).await;
}

fn store_presence(state: &AppState, user_id: Uuid, platform: Platform, presence: Presence) {
    let mut held = write_lock(&state.game_presence.by_user);
    let activity = held.entry(user_id).or_default();
    activity.set(platform, Some(presence));
}

fn remove_presence(state: &AppState, user_id: Uuid, platform: Platform) -> bool {
    let mut held = write_lock(&state.game_presence.by_user);

    let Some(activity) = held.get_mut(&user_id) else {
        return false;
    };

    let had_presence = activity.get(platform).is_some();
    activity.set(platform, None);
    had_presence
}
