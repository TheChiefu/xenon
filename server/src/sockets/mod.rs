//! Connected users: the sockets they hold, what is pushed to them, and what
//! others are told about them.

pub mod call;
pub mod chat;
pub mod events;

use uuid::Uuid;

use crate::shared::Status;
use crate::sockets::events::ServerEvent;
use crate::state::AppState;

/// Closes every chat and call socket a user holds
pub async fn disconnect(
    state: &AppState,
    user_id: Uuid,
) {
    // Sign the user's clients out before their sockets close
    chat::registry::inform_user(state, user_id, ServerEvent::SessionRevoked);
    chat::registry::remove(state, user_id);

    // Tell others the user is offline
    let event = ServerEvent::PresenceUpdated { user_id, status: Status::Offline, device: None };
    chat::registry::inform_shared_members(state, user_id, event).await;

    // Tell the room the user left its call
    if let Some(room_id) = call::registry::leave(state, user_id) {
        let event = ServerEvent::ParticipantLeft { room_id, user_id };
        chat::registry::broadcast(state, room_id, event).await;
    }
}
