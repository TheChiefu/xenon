//! What each linked account last reported, and who it is sent to.

use std::collections::BTreeMap;
use std::sync::RwLock;

use uuid::Uuid;

use crate::sockets::session::events::ServerEvent;
use crate::state::{self, appearance, AppState};
use crate::types::appearance::Appearance;
use crate::services::links::types::{Activity, Metadata, UserActivity};
use crate::types::platforms::Platform;
use crate::utils;

/// What each user's linked accounts last reported, keyed by user id
static REPORTED: RwLock<BTreeMap<Uuid, Activity>> = RwLock::new(BTreeMap::new());

/// Stores a reported platform metadata and sends it to everyone sharing a room with its owner,
/// or clears it while the owner appears Offline
pub async fn report_metadata(
    state: &AppState,
    user_id: Uuid,
    platform: Platform,
    metadata: Metadata,
) {
    // Offline users show no platform presence
    if appearance::get(&state.appearances, user_id) == Some(Appearance::Offline) {
        remove_metadata(user_id, platform);
        return;
    }

    store_metadata(user_id, platform, metadata.clone());
    inform_updated(state, user_id, platform, Some(metadata)).await;
}

/// Build the platform activity of every given member
pub fn snapshot(members: &[Uuid]) -> Vec<UserActivity> {
    let held = utils::read_lock(&REPORTED);

    let mut users = Vec::new();
    for member_id in members {
        let Some(activity) = held.get(member_id) else {
            continue;
        };

        users.push(UserActivity {
            user_id: *member_id,
            activity: activity.clone()
        });
    }

    users
}

/// Clear individual platform metadata
pub async fn clear_metadata(
    state: &AppState,
    user_id: Uuid,
    platform: Platform,
) {
    if remove_metadata(user_id, platform) {
        inform_updated(state, user_id, platform, None).await;
    }
}

/// Clear every platform metadata
pub async fn clear_all(
    state: &AppState,
    user_id: Uuid,
) {
    clear_metadata(state, user_id, Platform::Xbox).await;
    clear_metadata(state, user_id, Platform::Steam).await;
}

/// Send a user's platform presence to everyone sharing a room with them
async fn inform_updated(
    state: &AppState,
    user_id: Uuid,
    platform: Platform,
    metadata: Option<Metadata>,
) {
    let event = ServerEvent::PlatformUpdated {
        user_id,
        platform,
        metadata
    };
    state::inform_shared_members(state, user_id, event).await;
}

fn store_metadata(
    user_id: Uuid,
    platform: Platform,
    metadata: Metadata,
) {
    let mut held = utils::write_lock(&REPORTED);
    let activity = held.entry(user_id).or_default();
    activity.set(platform, Some(metadata));
}

fn remove_metadata(
    user_id: Uuid,
    platform: Platform,
) -> bool {
    let mut held = utils::write_lock(&REPORTED);

    let Some(activity) = held.get_mut(&user_id) else {
        return false;
    };

    let had_metadata = activity.get(platform).is_some();
    activity.set(platform, None);
    had_metadata
}
