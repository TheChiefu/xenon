//! One user's RTC socket, from joining a room's connections until leaving them

use std::sync::Arc;

use axum::extract::ws::{CloseFrame, Message, WebSocket};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::response::Response;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::api;
use crate::config;
use crate::error::AppError;
use crate::routes::{AuthUser, BEARER};
use crate::rtc::{self, connections};
use crate::rtc::events::{RtcClientEvent, RtcCloseCode, RtcServerEvent};
use crate::rtc::peer::{self, Peer};
use crate::sockets;
use crate::sockets::session::events::ServerEvent;
use crate::state::{self, AppState};
use crate::types::bytesize::KIBIBYTE;

/// Largest message a client may send over an RTC socket
const SIGNAL_SIZE_MAX: usize = 16 * KIBIBYTE as usize;

// ROUTING METHODS //

/// Upgrade an HTTP request into an RTC socket for a room
pub async fn ws_handler(
    AuthUser(user_id, ..): AuthUser,
    State(app_state): State<AppState>,
    Path(room_id): Path<Uuid>,
    ws: WebSocketUpgrade,
) -> Response {

    let handshake = ws.protocols([BEARER]).max_message_size(SIGNAL_SIZE_MAX);
    let start_socket = move |socket| {
        handle_socket(socket, user_id, room_id, app_state)
    };
    handshake.on_upgrade(start_socket)
}

// HELPER METHODS //

/// Run one user's RTC socket, from joining until leaving
async fn handle_socket(
    mut socket: WebSocket,
    user_id: Uuid,
    room_id: Uuid,
    app_state: AppState,
) {
    match api::rooms::access::get(&app_state.pool, room_id, user_id, user_id).await {
        Ok(_) => {}
        Err(AppError::Forbidden) => {
            close(&mut socket, RtcCloseCode::PolicyViolation).await;
            return;
        }
        Err(e) => {
            tracing::error!("could not read the access of {user_id} to {room_id}: {e}");
            close(&mut socket, RtcCloseCode::ServerError).await;
            return;
        }
    }

    let Some((peer, receiver)) = open(user_id, room_id, &app_state).await else {
        close(&mut socket, RtcCloseCode::NoPort).await;
        return;
    };

    // A user connected elsewhere leaves there first
    if let Some(previous) = connections::peer(&app_state.rtc, user_id) {
        rtc::leave(&app_state, &previous).await;
    }

    if let Err(code) = join(&app_state, &peer).await {
        if let Err(e) = peer.link.close().await {
            tracing::warn!("could not close the connection of {user_id}: {e}");
        }
        close(&mut socket, code).await;
        return;
    }

    let (mut outgoing, mut incoming) = socket.split();

    let code = tokio::select! {
        _ = write_events(&mut outgoing, receiver) => RtcCloseCode::Normal,
        _ = read_events(&mut incoming, &peer, &app_state) => RtcCloseCode::Normal,
        code = peer.signal.ended() => code
    };

    let frame = CloseFrame { code: code as u16, reason: "".into() };
    let _ = outgoing.send(Message::Close(Some(frame))).await;

    rtc::leave(&app_state, &peer).await;
}

/// Open a connection on the first port in the configured range that is free
async fn open(
    user_id: Uuid,
    room_id: Uuid,
    app_state: &AppState,
) -> Option<(Arc<Peer>, mpsc::UnboundedReceiver<String>)> {
    for port in config::get().sockets.udp_ports.clone() {
        let connections = app_state.rtc.clone();
        match peer::connect(user_id, room_id, port, connections).await {
            Ok(opened) => return Some(opened),
            Err(e) => tracing::debug!("could not open a connection for {user_id} on port {port}: {e}")
        }
    }

    tracing::error!("no port in sockets.udp_ports could be bound for {user_id}");
    None
}

/// Add a peer to its room and send everyone in it new offers
async fn join(
    app_state: &AppState,
    peer: &Arc<Peer>,
) -> Result<(), RtcCloseCode> {
    let lock = app_state.rtc.track_lock.lock().await;

    let others = connections::insert(&app_state.rtc, peer.clone())?;

    for other in &others {
        if let Err(e) = peer.forwarding.add_sender(other.user_id).await {
            tracing::error!("could not add {}'s tracks for {}: {e}", other.user_id, peer.user_id);
        }
        if let Err(e) = other.forwarding.add_sender(peer.user_id).await {
            tracing::error!("could not add {}'s tracks for {}: {e}", peer.user_id, other.user_id);
        }
        peer.keyframes.relay_sender(&peer.forwarding, other.user_id);
        other.keyframes.relay_sender(&other.forwarding, peer.user_id);
        other.negotiate().await;

        if connections::is_sharing(&app_state.rtc, other.user_id) {
            peer.signal.send(&RtcServerEvent::Screen { user_id: other.user_id, sharing: true });
        }
    }

    peer.negotiate().await;

    drop(lock);

    let event = ServerEvent::RtcJoined { room_id: peer.room_id, user_id: peer.user_id };
    state::broadcast(app_state, peer.room_id, event).await;

    Ok(())
}

/// Close a socket with a code
async fn close(
    socket: &mut WebSocket,
    code: RtcCloseCode,
) {
    let frame = CloseFrame { code: code as u16, reason: "".into() };
    let _ = socket.send(Message::Close(Some(frame))).await;
}

/// Write queued events through the socket
async fn write_events(
    socket: &mut SplitSink<WebSocket, Message>,
    mut receiver: mpsc::UnboundedReceiver<String>,
) {
    while let Some(text) = receiver.recv().await {
        if socket.send(Message::Text(text.into())).await.is_err() {
            break;
        }
    }
}

/// Apply what a client sends
async fn read_events(
    socket: &mut SplitStream<WebSocket>,
    peer: &Arc<Peer>,
    app_state: &AppState,
) {
    while let Some(event) = sockets::read::<RtcClientEvent>(socket, peer.user_id).await {
        match event {
            RtcClientEvent::Answer { description } => peer.answer(description).await,
            RtcClientEvent::Candidate { candidate } => {
                if let Err(e) = peer.link.add_candidate(candidate).await {
                    tracing::debug!("could not add a candidate from {}: {e}", peer.user_id);
                }
            }
            RtcClientEvent::Kick { user_id } => kick(app_state, peer, user_id).await,
            RtcClientEvent::Screen { sharing } => share(app_state, peer, sharing)
        }
    }
}

/// Tell everyone else in the peer's room whether the peer is sharing their screen
fn share(
    app_state: &AppState,
    peer: &Arc<Peer>,
    sharing: bool,
) {
    connections::set_sharing(&app_state.rtc, peer.user_id, sharing);

    let event = RtcServerEvent::Screen { user_id: peer.user_id, sharing };
    for other in connections::in_room(&app_state.rtc, peer.room_id) {
        if other.user_id != peer.user_id {
            other.signal.send(&event);
        }
    }
}

/// End another user's RTC socket in the peer's room when the peer may kick
async fn kick(
    app_state: &AppState,
    peer: &Arc<Peer>,
    target_id: Uuid,
) {
    match api::rooms::can_kick_from_call(&app_state.pool, peer.room_id, peer.user_id).await {
        Ok(true) => rtc::end_user(app_state, target_id, Some(peer.room_id), RtcCloseCode::Kicked),
        Ok(false) => {}
        Err(e) => tracing::error!("could not read whether {} may kick: {e}", peer.user_id)
    }
}
