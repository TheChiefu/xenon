use axum::extract::ws::{CloseFrame, Message, WebSocket};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::response::Response;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::config;
use crate::db;
use crate::error::Result;
use crate::routes::AuthUser;
use crate::shared::Permission;
use crate::sockets::call::registry::{self, JoinRefused};
use crate::sockets::chat;
use crate::sockets::events::ServerEvent;
use crate::sockets::events::call::{CallClientEvent, CallCloseCode, CallServerEvent, IceServer};
use crate::state::AppState;

// Socketing Methods //

/// Upgrades an HTTP request into a call socket for a room
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

/// Holds a user in a room's call for as long as the socket is open
async fn handle_socket(
    mut socket: WebSocket,
    user_id: Uuid,
    room_id: Uuid,
    state: AppState,
) {
    match can_connect(&state, room_id, user_id).await {
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

    let receiver = match registry::join(&state, room_id, user_id) {
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
    chat::registry::broadcast(&state, room_id, event).await;

    let (mut outgoing, mut incoming) = socket.split();

    send_joined(&state, room_id, &mut outgoing).await;

    // Either one finishing means the user has left the call
    tokio::select! {
        _ = send_signals(&mut outgoing, receiver) => {}
        _ = read_events(&mut incoming, user_id, &state) => {}
    }

    // Tell the room unless the call already ended
    if let Some(room_id) = registry::leave(&state, user_id) {
        let event = ServerEvent::ParticipantLeft { room_id, user_id };
        chat::registry::broadcast(&state, room_id, event).await;
    }
}

/// Reads whether a user holds `Connect` in a room
async fn can_connect(
    state: &AppState,
    room_id: Uuid,
    user_id: Uuid,
) -> Result<bool> {
    let mut conn = state.pool.acquire().await?;
    let permissions = db::effective_permissions(&mut conn, room_id, user_id).await?;

    // A user with no access to the room has no permissions
    match permissions {
        Some(permissions) => Ok(permissions.contains(&Permission::Connect)),
        None => Ok(false)
    }
}

/// Closes a socket with the code for why
async fn close(
    socket: &mut WebSocket,
    code: CallCloseCode,
) {
    let frame = CloseFrame { code: code as u16, reason: "".into() };
    let _ = socket.send(Message::Close(Some(frame))).await;
}

/// Writes who is already in the call, and the ICE servers the browser uses
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
        participants: registry::participants(state, room_id),
        ice_servers
    };

    let payload = match serde_json::to_string(&event) {
        Ok(payload) => payload,
        Err(e) => {
            tracing::error!("failed to serialize call joined event: {e}");
            return;
        }
    };

    let _ = socket.send(Message::Text(payload.into())).await;
}

/// Writes queued signals through the socket until the queue closes or a write fails
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

/// Applies what a client sends until it closes the connection
async fn read_events(
    socket: &mut SplitStream<WebSocket>,
    user_id: Uuid,
    state: &AppState,
) {
    while let Some(Ok(message)) = socket.next().await {

        // Only text frames carry an event
        let Message::Text(text) = message else {
            continue;
        };

        let event = match serde_json::from_str::<CallClientEvent>(&text) {
            Ok(event) => event,
            Err(e) => {
                tracing::debug!("call socket for {user_id} sent an unreadable event: {e}");
                continue;
            }
        };

        match event {
            CallClientEvent::Signal { to, payload } => {
                registry::forward(state, user_id, to, payload);
            }
        }
    }
}
