//! What each linked account last reported, and who it is sent to.

use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use uuid::Uuid;

use crate::db;
use crate::error::Result;
use crate::shared::Status;
use crate::sockets::events::{GameActivity, GamePresence, ServerEvent, UserGamePresence};
use crate::sockets::chat::registry;
use crate::state::AppState;

/// What each user's linked account last reported, keyed by user id
#[derive(Clone, Default)]
pub struct ReportedGames {
    by_user: Arc<RwLock<HashMap<Uuid, GameActivity>>>
}

impl ReportedGames {

    /// Read what each linked account last reported (RwLock)
    fn read_map(&self) -> RwLockReadGuard<'_, HashMap<Uuid, GameActivity>> {
        match self.by_user.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::warn!("game_presence lock was poisoned, recovering");
                poisoned.into_inner()
            }
        }
    }

    /// Write what each linked account last reported (RwLock)
    fn write_map(&self) -> RwLockWriteGuard<'_, HashMap<Uuid, GameActivity>> {
        match self.by_user.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::warn!("game_presence lock was poisoned, recovering");
                poisoned.into_inner()
            }
        }
    }
}

/// Stores a reported game presence and sends it to everyone sharing a room with its owner.
pub async fn on_report(
    state: &AppState,
    user_id: Uuid,
    game: GameActivity,
) {
    let declared = match registry::status_of(state, user_id) {
        Some(entry) => Some(entry.status),
        None => None
    };

    // Offline users have no game presence, remove entry
    if declared == Some(Status::Offline) {
        state.game_presence.write_map().remove(&user_id);
        return;
    }

    match game.status {
        GamePresence::Offline => state.game_presence.write_map().remove(&user_id),
        _ => state.game_presence.write_map().insert(user_id, game.clone())
    };

    let event = ServerEvent::GamePresenceUpdated { user_id, game };
    registry::inform_shared_members(state, user_id, event).await;
}

/// Builds the game presence list a connecting user is sent. It contains
/// everyone they share a room with who is on a linked account.
pub async fn snapshot(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<UserGamePresence>> {

    // The members whose game presence this user may see
    let mut conn = state.pool.acquire().await?;
    let members = db::shared_room_member_ids(&mut conn, user_id).await?;

    let held = state.game_presence.read_map();

    let mut users = Vec::new();
    for member_id in members {
        let Some(game) = held.get(&member_id) else {
            continue;
        };

        users.push(UserGamePresence { user_id: member_id, game: game.clone() });
    }

    Ok(users)
}

/// Removes a user's game presence and sends everyone sharing a room with them
/// an Offline update for that account.
pub async fn clear(state: &AppState, user_id: Uuid) {
    let removed = state.game_presence.write_map().remove(&user_id);
    let Some(game) = removed else {
        return;
    };

    let event = ServerEvent::GamePresenceUpdated {
        user_id,
        game: GameActivity {
            platform: game.platform,
            status: GamePresence::Offline,
            title: None,
            detail: None
        }
    };
    registry::inform_shared_members(state, user_id, event).await;
}
