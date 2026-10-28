//! Connected users: the sockets they hold, what is pushed to them, and what
//! others are told about them.

pub mod call;
pub mod session;

use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use serde::Serialize;
use uuid::Uuid;

/// Serialize an event for the wire, or None if serialization fails
pub fn encode<T: Serialize>(event: &T) -> Option<String> {
    match serde_json::to_string(event) {
        Ok(payload) => Some(payload),
        Err(e) => {
            tracing::error!("failed to serialize socket event: {e}");
            None
        }
    }
}

/// Send an event, returns whether the socket is still open
pub async fn send<T: Serialize>(
    socket: &mut SplitSink<WebSocket, Message>,
    event: &T,
) -> bool {
    let Some(payload) = encode(event) else {
        return true;
    };

    socket.send(Message::Text(payload.into())).await.is_ok()
}

/// Read the next event, or None once the socket closes
pub async fn read<T: DeserializeOwned>(
    socket: &mut SplitStream<WebSocket>,
    user_id: Uuid,
) -> Option<T> {
    while let Some(Ok(message)) = socket.next().await {

        // Only text frames carry an event
        let Message::Text(text) = message else {
            continue;
        };

        match serde_json::from_str::<T>(&text) {
            Ok(event) => return Some(event),
            Err(e) => {
                tracing::debug!("socket for {user_id} sent an unreadable event: {e}");
            }
        }
    }

    None
}
