//! Voice and video in rooms over WebRTC, relayed through the server
//!
//! https://www.w3.org/TR/webrtc/

pub mod events;
pub mod registry;
pub mod socket;
mod handler;
mod media;
mod peer;

use std::sync::Arc;

use uuid::Uuid;

use crate::rtc::events::RtcCloseCode;
use crate::rtc::peer::Peer;
use crate::sockets::session::events::ServerEvent;
use crate::state::{self, AppState};

// PUBLIC METHODS //

/// Remove a peer and tell its room the user left
pub async fn leave(
    state: &AppState,
    peer: &Arc<Peer>,
) {
    let lock = state.rtc.track_lock.lock().await;
    let removed = remove_peer(state, peer).await;
    drop(lock);

    if removed {
        let event = ServerEvent::RtcLeft { room_id: peer.room_id, user_id: peer.user_id };
        state::broadcast(state, peer.room_id, event).await;
    }
}

/// End a user's RTC socket when they are connected in the given room
pub fn end_in_room(
    state: &AppState,
    room_id: Uuid,
    user_id: Uuid,
    code: RtcCloseCode,
) {
    if let Some(peer) = registry::peer(&state.rtc, user_id) {
        if peer.room_id == room_id {
            peer.end(code);
        }
    }
}

/// End a user's RTC socket in whichever room they are connected
pub fn end_user(
    state: &AppState,
    user_id: Uuid,
) {
    if let Some(peer) = registry::peer(&state.rtc, user_id) {
        peer.end(RtcCloseCode::Normal);
    }
}

/// End every RTC socket in a room
pub fn end_room(
    state: &AppState,
    room_id: Uuid,
) {
    for peer in registry::peers(&state.rtc, room_id) {
        peer.end(RtcCloseCode::Normal);
    }
}

// HELPER METHODS //

/// Remove a peer and its tracks from the others in the room, returning whether it
/// was still connected. The caller holds the track lock.
async fn remove_peer(
    state: &AppState,
    peer: &Arc<Peer>,
) -> bool {
    let Some(remaining) = registry::remove(&state.rtc, peer) else {
        return false;
    };

    for other in &remaining {
        other.remove_sender(peer.user_id).await;
        other.negotiate().await;
    }

    peer.end(RtcCloseCode::Normal);
    peer.close().await;

    true
}
