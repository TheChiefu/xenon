//! One client's WebSocket, from the handshake until it closes.

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::Response;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::routes::AuthUser;
use crate::sockets;
use crate::sockets::session::events::{ClientEvent, ServerEvent};
use crate::sockets::session::registry::{self, SocketDevice};
use crate::state::status;
use crate::state::{self, AppState};
use crate::types::status::{Appearance, Device};

// Socketing Methods //

/// Query string the handshake carries, read before the upgrade while the
/// request is still an ordinary GET.
#[derive(Deserialize)]
pub struct ConnectQuery {
    #[serde(default)]
    pub device: Option<Device>,
    pub status: Appearance,
}

// Upgrade an HTTP request into a WebSocket
pub async fn ws_handler(
    AuthUser(user_id, ..): AuthUser,
    State(state): State<AppState>,
    Query(query): Query<ConnectQuery>,
    ws: WebSocketUpgrade,
) -> Response {

    let handshake = ws.protocols(["Bearer"]);
    let start_socket = move |socket| {
        handle_socket(socket, user_id, query.device, query.status, state)
    };
    handshake.on_upgrade(start_socket)
}

// Helper Methods //

// Push server events to one connection until it closes
async fn handle_socket(
    socket: WebSocket,
    user_id: Uuid,
    device: Option<Device>,
    status: Appearance,
    state: AppState,
) {
    // Identifies this socket's entry in the user's device list
    let socket_id = registry::next_socket_id();
    let device = match device {
        Some(device) => Some(SocketDevice { socket_id, device }),
        None => None
    };

    let already_connected = registry::is_connected(&state.registry, user_id);
    let mut receiver = registry::subscribe(&state.registry, user_id, device);

    status::connected(&state, user_id, status, already_connected).await;

    // A single connection cannot be read and written at once, so it is split
    // into two parts that can be
    let (mut outgoing, mut incoming) = socket.split();

    // Send snapshots on connect
    send_snapshots(&state, user_id, &mut outgoing).await;

    // Either one finishing means the connection is over
    tokio::select! {
        _ = send_events(&mut outgoing, &mut receiver, user_id, &state) => {}
        _ = read_events(&mut incoming, user_id, &state) => {}
    }

    registry::unsubscribe(&state.registry, user_id, socket_id, receiver);
    status::disconnected(&state, user_id).await;
}

// Write every snapshot a viewer is sent through the socket
async fn send_snapshots(
    state: &AppState,
    viewer_id: Uuid,
    socket: &mut SplitSink<WebSocket, Message>,
) {
    for event in state::snapshots(state, viewer_id).await {
        sockets::send(socket, &event).await;
    }
}


// Read/write events from a user's channel through the socket.
// Ends when the channel closes or a write fails.
async fn send_events(
    socket: &mut SplitSink<WebSocket, Message>,
    receiver: &mut broadcast::Receiver<String>,
    user_id: Uuid,
    app_state: &AppState,
) {
    loop {
        let text = match receiver.recv().await {
            Ok(text) => text,

            // The channel keeps a fixed number of recent events and overwrites
            // the earliest to make room. This connection fell far enough behind
            // that events it had not read were overwritten, so the client is
            // told to re-read over HTTP. Its place in the channel moves forward
            // to the earliest event still kept.
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                tracing::warn!("socket for {user_id} skipped {skipped} events");

                // Send snapshots on lag to catch up
                send_snapshots(app_state, user_id, socket).await;

                if !sockets::send(socket, &ServerEvent::Resync).await {
                    break;
                }
                continue;
            }

            Err(broadcast::error::RecvError::Closed) => break,
        };

        if socket.send(Message::Text(text.into())).await.is_err() {
            break;
        }
    }
}

// Apply what a client sends until it closes the connection
async fn read_events(
    socket: &mut SplitStream<WebSocket>,
    user_id: Uuid,
    state: &AppState,
) {
    while let Some(event) = sockets::read::<ClientEvent>(socket, user_id).await {
        match event {
            ClientEvent::Status { status } => {
                status::set(state, user_id, status);
                status::announce(state, user_id, status).await;
            }
        }
    }
}


