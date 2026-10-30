//! HTTP handlers for messages.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::api;
use crate::api::messages::{Cursor, Posted};
use crate::api::messages::attachments::Attachment;
use crate::error::{AppError, Result};
use crate::routes::AuthUser;
use crate::services;
use crate::sockets::session::events::ServerEvent;
use crate::sockets::session::registry;
use crate::state::{self, AppState};
use crate::types::messages::{AttachmentInfo, AttachmentRequest, MessageInfo};

// REQUESTS //

#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/messages.ts"))]
pub struct RequestMessageSend {
    pub body: Option<String>,
    pub client_nonce: String,
    #[serde(default)]
    pub attachments: Vec<AttachmentRequest>
}

#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/messages.ts"))]
pub struct RequestMessageEdit {
    pub body: Option<String>
}

// RESPONSES //

#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/messages.ts"))]
pub struct RespondMessageEdit {
    pub edited_at: i64
}

// DATA STRUCTS //

#[derive(Deserialize)]
pub struct FetchQuery {
    pub after: Option<i64>,
    pub before: Option<i64>,
}

// ROUTING METHODS //

/// Post a message to a room
pub async fn post(
    AuthUser(author_id, ..): AuthUser,
    State(app_state): State<AppState>,
    Path(room_id): Path<Uuid>,
    Json(body): Json<RequestMessageSend>,
) -> Result<(StatusCode, Json<MessageInfo>)> {

    // Check for malformed nonce (check if hex)
    let bytes = match hex::decode(&body.client_nonce) {
        Ok(bytes) => bytes,
        Err(_) => return Err(AppError::Validation("client_nonce must be hex".to_string()))
    };

    // Check nonce byte count
    let nonce: [u8; 16] = match bytes.try_into() {
        Ok(array) => array,
        Err(_) => return Err(AppError::Validation("client_nonce must be 16 bytes".to_string()))
    };

    // Attempt to post a message
    let (result, files) = api::messages::post(
        &app_state.pool,
        room_id,
        author_id,
        body.body.as_deref(),
        nonce,
        &body.attachments
    ).await?;

    let (status, message) = match result {
        Posted::Created(msg) => (StatusCode::CREATED, msg),
        Posted::Duplicate(msg) => (StatusCode::OK, msg),
    };

    let attachments = attachment_infos(files);
    let response = MessageInfo::new(message, attachments);

    // If message is posted, broadcast to all subscribed users in room
    if status == StatusCode::CREATED {
        let msg_event = ServerEvent::Message { room_id, message: response.clone() };
        state::broadcast(&app_state, room_id, msg_event).await;

        // Notify users who opted in
        if let Err(e) = notify_users(&app_state, room_id, &response).await {
            tracing::error!("could not notify members of {room_id}: {e}");
        }
    }

    // Message is duplicate (no broadcast)
    Ok((status, Json(response)))
}

/// Fetch one page of a room's messages
pub async fn fetch(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Path(room_id): Path<Uuid>,
    Query(query): Query<FetchQuery>,
) -> Result<Json<Vec<MessageInfo>>> {

    // Get http query (?before, ?after) and convert it into a cursor
    let cursor = match (query.after, query.before) {
        (None, None) => Cursor::Latest,
        (Some(seq), None) => Cursor::After(seq),
        (None, Some(seq)) => Cursor::Before(seq),
        (Some(_), Some(_)) => {
            return Err(AppError::Validation("cannot use before/after".to_string()))
        }
    };

    // Attempt to fetch messages
    let result = api::messages::fetch(&pool, room_id, user_id, cursor).await?;

    // Convert returned vector of internal messages into vector of HTTP message responses
    let mut response = Vec::with_capacity(result.len());
    for (message, files) in result {
        let attachments = attachment_infos(files);

        response.push(MessageInfo::new(message, attachments));
    }

    Ok(Json(response))
}

/// Get a message
pub async fn get(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Path(message_id): Path<Uuid>,
) -> Result<Json<MessageInfo>> {

    let (message, files) = api::messages::get(&pool, message_id, user_id).await?;

    let attachments = attachment_infos(files);

    Ok(Json(MessageInfo::new(message, attachments)))
}

/// Delete a message
pub async fn delete(
    AuthUser(caller_id, ..): AuthUser,
    State(app_state): State<AppState>,
    Path(message_id): Path<Uuid>,
) -> Result<StatusCode> {

    let room_id = api::messages::delete(&app_state.pool, message_id, caller_id).await?;

    let event = ServerEvent::MessageDeleted { room_id, message_id };
    state::broadcast(&app_state, room_id, event).await;

    Ok(StatusCode::NO_CONTENT)
}

/// Edit a message
pub async fn update(
    AuthUser(caller_id, ..): AuthUser,
    State(app_state): State<AppState>,
    Path(message_id): Path<Uuid>,
    Json(request): Json<RequestMessageEdit>,
) -> Result<Json<RespondMessageEdit>> {

    let result = api::messages::edit(
        &app_state.pool,
        message_id,
        caller_id,
        request.body.as_deref(),
    ).await?;

    let event = ServerEvent::MessageEdited {
        room_id: result.room_id,
        message_id,
        body: request.body,
        edited_at: result.edited_at
    };
    state::broadcast(&app_state, result.room_id, event).await;

    Ok(Json(RespondMessageEdit { edited_at: result.edited_at }))
}

// HELPER METHODS //

/// Convert stored attachments into response form
fn attachment_infos(files: Vec<Attachment>) -> Vec<AttachmentInfo> {
    let mut infos = Vec::with_capacity(files.len());
    for file in files {
        infos.push(AttachmentInfo::from(file));
    }
    infos
}

/// Send notification event to room members
async fn notify_users(
    app_state: &AppState,
    room_id: Uuid,
    message: &MessageInfo,
) -> Result<()> {

    // Skip notifying when the message has no author
    let Some(author_id) = message.author_id else {
        return Ok(());
    };

    let notification = api::messages::notifications::notify(
        &app_state.pool,
        room_id,
        author_id,
        message.body.as_deref()
    ).await?;

    // Notify offline users
    services::notify_offline(app_state, &notification);

    // Create notification server event
    let event = ServerEvent::Notification {
        room_id,
        room_name: notification.room_name,
        author: notification.author,
        body: notification.body
    };

    // Notify connections users
    registry::inform_users(&app_state.registry, &notification.recipients, event);

    Ok(())
}
