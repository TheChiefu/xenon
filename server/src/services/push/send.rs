use std::time::Duration;

use reqwest::StatusCode;
use serde::Serialize;
use sqlx::SqlitePool;
use uuid::Uuid;
use web_push_native::jwt_simple::algorithms::ES256KeyPair;
use web_push_native::p256::elliptic_curve::sec1::ToEncodedPoint;
use web_push_native::p256::{PublicKey, SecretKey};
use web_push_native::{Auth, WebPushBuilder};

use crate::config;
use crate::error::Result;
use crate::services::CLIENT;
use crate::services::push::store;
use crate::services::push::store::Subscription;
use crate::sockets::session::registry;
use crate::state::AppState;

/// Notification contents as JSON, read by a service worker
#[derive(Serialize)]
pub struct Payload {
    pub room_id: Uuid,
    pub room: String,
    pub author: String,
    pub body: String,

    /// Alerts again rather than replacing quietly
    pub renotify: bool
}

/// Generates a keypair if there is none yet
pub async fn ensure_keypair(pool: &SqlitePool) -> Result<()> {
    if store::keypair(pool).await?.is_some() {
        return Ok(());
    }

    let secret = SecretKey::random(&mut rand_core::OsRng);
    let public_key = secret.public_key().to_encoded_point(false);

    store::set_keypair(pool, public_key.as_bytes(), secret.to_bytes().as_slice()).await
}

/// Cuts a message body to fit in a push payload
pub fn shorten(body: &str) -> String {

    // Keeps the encrypted payload under 3800 bytes
    let max_chars = 500;

    match body.char_indices().nth(max_chars) {
        Some((index, _)) => format!("{}...", &body[..index]),
        None => body.to_string()
    }
}

/// Spawns a task notifying the given users who hold no socket
pub fn notify_offline(app_state: &AppState, payload: Payload, user_ids: &[Uuid]) {
    let offline = registry::offline_users(&app_state.registry, user_ids);
    if offline.is_empty() {
        return;
    }

    let push = match &config::get().push {
        Some(push) => push,
        None => return
    };

    let pool = app_state.pool.clone();

    tokio::spawn(async move {
        if let Err(e) = notify(&pool, push, &payload, &offline).await {
            tracing::error!("could not send notifications: {e}");
        }
    });
}

/// Posts one payload per subscription
pub async fn notify(
    pool: &SqlitePool,
    push: &config::Push,
    payload: &Payload,
    user_ids: &[Uuid],
) -> Result<()> {
    let subscriptions = store::for_users(pool, user_ids).await?;
    if subscriptions.is_empty() {
        return Ok(());
    }

    let Some(keys) = store::keypair(pool).await? else {
        return Ok(());
    };

    let key = match ES256KeyPair::from_bytes(&keys.private_key) {
        Ok(key) => key,
        Err(e) => {
            tracing::error!("stored VAPID key is unusable: {e}");
            return Ok(());
        }
    };

    let encoded = serde_json::to_vec(payload).unwrap_or_default();

    for subscription in subscriptions {

        // Encrypt payload
        let Some(request) = encrypt(&subscription, &encoded, &key, push) else {
            continue;
        };

        // Post to a push service
        let response = match CLIENT.execute(request).await {
            Ok(response) => response,
            Err(e) => {
                tracing::trace!("Could not POST to service: {e}");
                continue;
            }
        };

        // Delete subscriptions if the response is an error
        if response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::GONE {
            store::remove_unreachable(pool, &subscription.endpoint).await?;
        }
    }

    Ok(())
}

/// Builds one encrypted request, signed with the VAPID key
fn encrypt(
    subscription: &Subscription,
    payload: &[u8],
    key: &ES256KeyPair,
    push: &config::Push,
) -> Option<reqwest::Request> {
    let endpoint = subscription.endpoint.parse().ok()?;
    let subscriber = PublicKey::from_sec1_bytes(&subscription.p256dh).ok()?;

    if subscription.auth.len() != Auth::default().len() {
        return None;
    }

    let request = WebPushBuilder::new(
        endpoint,
        subscriber,
        Auth::clone_from_slice(&subscription.auth)
    )
        .with_valid_duration(Duration::from_secs(push.ttl.into()))
        .with_vapid(key, &push.subject)
        .build(payload.to_vec())
        .ok()?;

    reqwest::Request::try_from(request).ok()
}
