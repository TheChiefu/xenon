//! What each linked account last reported, and who it is sent to.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use uuid::Uuid;

use crate::sockets::session::events::ServerEvent;
use crate::state::{self, status, AppState};
use crate::types::platforms::{Activity, Metadata, Platform, UserActivity};
use crate::types::status::Appearance;
use crate::utils;

/// What each user's linked accounts last reported, keyed by user id
#[derive(Clone, Default)]
pub struct ReportedPlatforms {
    by_user: Arc<RwLock<HashMap<Uuid, Activity>>>
}

/// Stores a reported platform metadata and sends it to everyone sharing a room with its owner
pub async fn report_metadata(
    state: &AppState,
    user_id: Uuid,
    platform: Platform,
    metadata: Metadata,
) {
    // Offline users show no platform presence
    if status::get(&state.statuses, user_id) == Some(Appearance::Offline) {
        remove_metadata(&state.platforms, user_id, platform);
        return;
    }

    store_metadata(&state.platforms, user_id, platform, metadata.clone());
    inform_updated(state, user_id, platform, Some(metadata)).await;
}

/// Build the platform activity of every given member
pub fn snapshot(
    platforms: &ReportedPlatforms,
    members: &[Uuid],
) -> Vec<UserActivity> {
    let held = utils::read_lock(&platforms.by_user);

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
    if remove_metadata(&state.platforms, user_id, platform) {
        inform_updated(state, user_id, platform, None).await;
    }
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
    platforms: &ReportedPlatforms,
    user_id: Uuid,
    platform: Platform,
    metadata: Metadata,
) {
    let mut held = utils::write_lock(&platforms.by_user);
    let activity = held.entry(user_id).or_default();
    activity.set(platform, Some(metadata));
}

fn remove_metadata(
    platforms: &ReportedPlatforms,
    user_id: Uuid,
    platform: Platform,
) -> bool {
    let mut held = utils::write_lock(&platforms.by_user);

    let Some(activity) = held.get_mut(&user_id) else {
        return false;
    };

    let had_metadata = activity.get(platform).is_some();
    activity.set(platform, None);
    had_metadata
}
