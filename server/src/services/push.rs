//! Web push notifications for users without an open socket

pub mod routes;
pub mod send;
pub mod store;

use uuid::Uuid;

use crate::config;
use crate::error::Result;
use crate::state::AppState;
use crate::types::messages::Notification;

/// Attempt to start push (when enabled)
pub async fn start(state: &AppState) -> Result<()> {
    if config::get().push.is_none() {
        return Ok(());
    }

    // Ensure VAPID keys exist for browsers to subscribe against
    send::ensure_keypair(&state.pool).await
}

/// Push a posted message to the offline users among the recipients
pub fn notify_offline(state: &AppState, notification: &Notification) {
    let payload = send::Payload {
        room_id: notification.room_id,
        room: notification.room_name.clone(),
        author: notification.author.clone(),
        body: send::shorten(&notification.body),
        renotify: false
    };

    send::notify_offline(state, payload, &notification.recipients);
}

/// Remove every push subscription a user has
pub async fn remove_subscriptions(state: &AppState, user_id: Uuid) -> Result<()> {
    store::remove_all(&state.pool, user_id).await
}
