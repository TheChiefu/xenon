use axum::extract::ws::{CloseFrame, Message, WebSocket};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::response::Response;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::api::rooms;
use crate::config;
use crate::routes::AuthUser;
use crate::sockets;
use crate::sockets::call::events::{CallClientEvent, CallCloseCode, CallServerEvent, IceServer};
use crate::sockets::call::registry::{self, JoinRefused};
use crate::sockets::session::events::ServerEvent;
use crate::state::{self, AppState};

// Socketing Methods //

// Upgrade an HTTP request into a call socket for a room
pub async fn ws_handler(
    AuthUser(user_id, ..): AuthUser,
    State(state): State<AppState>,
    Path(room_id): Path<Uuid>,
    ws: WebSocketUpgrade,
) -> Response {

    let handshake = ws.protocols(["Bearer"]);
    let start_socket = move |socket| {
        handle_socket(socket, user_id, room_id, state)
    };
    handshake.on_upgrade(start_socket)
}

// Helper Methods //

// Socket information of a user in a room's call
async fn handle_socket(
    mut socket: WebSocket,
    user_id: Uuid,
    room_id: Uuid,
    state: AppState,
) {
    match rooms::can_connect(&state.pool, room_id, user_id).await {
        Ok(true) => {}
        Ok(false) => {
            close(&mut socket, CallCloseCode::NoConnect).await;
            return;
        }
        Err(e) => {
            tracing::error!("could not read the permissions of {user_id} in {room_id}: {e}");
            close(&mut socket, CallCloseCode::ServerError).await;
            return;
        }
    }

    let receiver = match registry::join(&state.calls, room_id, user_id) {
        Ok(receiver) => receiver,
        Err(JoinRefused::AlreadyInCall) => {
            close(&mut socket, CallCloseCode::AlreadyInCall).await;
            return;
        }
        Err(JoinRefused::Full) => {
            close(&mut socket, CallCloseCode::CallFull).await;
            return;
        }
    };

    let event = ServerEvent::ParticipantJoined { room_id, user_id };
    state::broadcast(&state, room_id, event).await;

    let (mut outgoing, mut incoming) = socket.split();

    send_joined(&state, room_id, &mut outgoing).await;

    // Either one finishing means the user has left the call
    tokio::select! {
        _ = send_signals(&mut outgoing, receiver) => {}
        _ = read_events(&mut incoming, user_id, &state) => {}
    }

    state::leave_call(&state, user_id).await;
}

// Close socket with the code for why
async fn close(
    socket: &mut WebSocket,
    code: CallCloseCode,
) {
    let frame = CloseFrame { code: code as u16, reason: "".into() };
    let _ = socket.send(Message::Close(Some(frame))).await;
}

// Write who is already in the call, and the ICE servers the browser uses
async fn send_joined(
    state: &AppState,
    room_id: Uuid,
    socket: &mut SplitSink<WebSocket, Message>,
) {
    let mut ice_servers = Vec::new();
    if let Some(url) = &config::get().calls.stun_url {
        ice_servers.push(IceServer { urls: url.clone() });
    }

    let event = CallServerEvent::CallJoined {
        participants: registry::participants(&state.calls, room_id),
        ice_servers
    };

    sockets::send(socket, &event).await;
}

// Write queued signals through the socket
async fn send_signals(
    socket: &mut SplitSink<WebSocket, Message>,
    mut receiver: mpsc::Receiver<String>,
) {
    while let Some(text) = receiver.recv().await {
        if socket.send(Message::Text(text.into())).await.is_err() {
            break;
        }
    }
}

// Apply what a client sends
async fn read_events(
    socket: &mut SplitStream<WebSocket>,
    user_id: Uuid,
    state: &AppState,
) {
    while let Some(event) = sockets::read::<CallClientEvent>(socket, user_id).await {
        match event {
            CallClientEvent::Signal { to, payload } => {
                registry::forward(&state.calls, user_id, to, payload);
            }
        }
    }
}
