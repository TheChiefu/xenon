//! Voice and video in rooms over WebRTC, relayed through the server
//!
//! https://www.w3.org/TR/webrtc/

pub mod connections;
pub mod events;
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

/// Remove a peer, take its tracks from the others in the room, and tell the room it left
pub async fn leave(
    state: &AppState,
    peer: &Arc<Peer>,
) {
    let lock = state.rtc.track_lock.lock().await;

    let Some(remaining) = connections::remove(&state.rtc, peer) else {
        return;
    };

    for other in &remaining {
        other.remove_sender(peer.user_id).await;
        other.negotiate().await;
    }

    drop(lock);

    peer.end(RtcCloseCode::Normal);
    peer.close().await;

    let event = ServerEvent::RtcLeft { room_id: peer.room_id, user_id: peer.user_id };
    state::broadcast(state, peer.room_id, event).await;
}

/// End a user's RTC socket, only when they are connected in the given room if one is given
pub fn end_user(
    state: &AppState,
    user_id: Uuid,
    room_id: Option<Uuid>,
    code: RtcCloseCode,
) {
    let Some(peer) = connections::peer(&state.rtc, user_id) else {
        return;
    };

    if room_id.is_none_or(|room_id| room_id == peer.room_id) {
        peer.end(code);
    }
}

/// End every RTC socket in a room
pub fn end_room(
    state: &AppState,
    room_id: Uuid,
) {
    for peer in connections::in_room(&state.rtc, room_id) {
        peer.end(RtcCloseCode::Normal);
    }
}
