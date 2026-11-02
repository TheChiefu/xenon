//! One user's RTC socket, from joining a room's connections until leaving them

use std::sync::Arc;

use axum::extract::ws::{CloseFrame, Message, WebSocket};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::response::Response;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use url::Url;
use uuid::Uuid;

use crate::api;
use crate::config;
use crate::error::AppError;
use crate::routes::{AuthUser, BEARER};
use crate::rtc::{self, registry};
use crate::rtc::events::{RtcClientEvent, RtcCloseCode};
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

    let Some(public_ips) = public_ips().await else {
        tracing::error!("could not resolve a public IPv4 address from network.public_url");
        close(&mut socket, RtcCloseCode::ServerError).await;
        return;
    };

    let Some(port) = registry::take_port(&app_state.rtc) else {
        close(&mut socket, RtcCloseCode::NoPort).await;
        return;
    };

    let connected = peer::connect(user_id, room_id, port, public_ips, app_state.rtc.clone()).await;
    let (peer, receiver) = match connected {
        Ok(connected) => connected,
        Err(e) => {
            tracing::error!("could not open a connection for {user_id} on port {port}: {e}");
            registry::release_port(&app_state.rtc, port);
            close(&mut socket, RtcCloseCode::ServerError).await;
            return;
        }
    };

    if !join(&app_state, &peer, port).await {
        peer.close().await;
        registry::release_port(&app_state.rtc, port);
        close(&mut socket, RtcCloseCode::Full).await;
        return;
    }

    let (mut outgoing, mut incoming) = socket.split();

    let code = tokio::select! {
        _ = write_events(&mut outgoing, receiver) => RtcCloseCode::Normal,
        _ = read_events(&mut incoming, &peer, &app_state) => RtcCloseCode::Normal,
        code = peer.ended() => code
    };

    let frame = CloseFrame { code: code as u16, reason: "".into() };
    let _ = outgoing.send(Message::Close(Some(frame))).await;

    rtc::leave(&app_state, &peer).await;
}

/// Add a peer to its room, moving the user out of any room they were connected in.
/// Returns false when the room is full.
async fn join(
    app_state: &AppState,
    peer: &Arc<Peer>,
    port: u16,
) -> bool {
    let lock = app_state.rtc.track_lock.lock().await;

    let mut moved_from = None;
    if let Some(previous) = registry::peer(&app_state.rtc, peer.user_id) {
        if rtc::remove_peer(app_state, &previous).await {
            moved_from = Some(previous.room_id);
        }
    }

    let Some(others) = registry::insert(&app_state.rtc, peer.clone(), port) else {
        drop(lock);
        announce_left(app_state, moved_from, peer.user_id).await;
        return false;
    };

    for other in &others {
        if let Err(e) = peer.add_sender(other.user_id).await {
            tracing::error!("could not add {}'s tracks for {}: {e}", other.user_id, peer.user_id);
        }
        if let Err(e) = other.add_sender(peer.user_id).await {
            tracing::error!("could not add {}'s tracks for {}: {e}", peer.user_id, other.user_id);
        }
        other.negotiate().await;
    }

    peer.negotiate().await;

    drop(lock);

    announce_left(app_state, moved_from, peer.user_id).await;

    let event = ServerEvent::RtcJoined { room_id: peer.room_id, user_id: peer.user_id };
    state::broadcast(app_state, peer.room_id, event).await;

    true
}

/// Tell a room the user moved out of that they left
async fn announce_left(
    app_state: &AppState,
    room_id: Option<Uuid>,
    user_id: Uuid,
) {
    if let Some(room_id) = room_id {
        let event = ServerEvent::RtcLeft { room_id, user_id };
        state::broadcast(app_state, room_id, event).await;
    }
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
            RtcClientEvent::Candidate { candidate } => peer.add_candidate(candidate).await,
            RtcClientEvent::Kick { user_id } => kick(app_state, peer, user_id).await
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
        Ok(true) => rtc::end_in_room(app_state, peer.room_id, target_id, RtcCloseCode::Kicked),
        Ok(false) => {}
        Err(e) => tracing::error!("could not read whether {} may kick: {e}", peer.user_id)
    }
}

/// The IPv4 addresses `network.public_url` resolves to, empty when it is unset.
/// Fails when it is set but does not resolve to an IPv4 address.
async fn public_ips() -> Option<Vec<String>> {
    let Some(public_url) = &config::get().network.public_url else {
        return Some(Vec::new());
    };

    let url = Url::parse(public_url).ok()?;
    let host = url.host_str()?;
    let addresses = tokio::net::lookup_host((host, 0)).await.ok()?;

    let mut ips = Vec::new();
    for address in addresses {
        if address.is_ipv4() {
            ips.push(address.ip().to_string());
        }
    }

    if ips.is_empty() {
        return None;
    }

    Some(ips)
}
