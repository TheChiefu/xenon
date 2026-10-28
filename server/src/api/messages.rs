//! The `messages` table

pub mod attachments;
pub mod notifications;
pub mod search;

use uuid::Uuid;

use crate::api::messages::attachments::Attachment;
use crate::config;
use crate::db;
use crate::error::{AppError, Result};
use crate::types::messages::{AttachmentInfo, AttachmentRequest, MessageInfo};
use crate::types::rooms::Permission;
use crate::utils;
use crate::validate;

// Data Structs //

/// A `messages` row
#[derive(sqlx::FromRow)]
pub struct Message {
    pub seq: i64,
    pub id: Uuid,
    pub room_id: Uuid,
    pub author_id: Option<Uuid>,
    pub body: Option<String>,
    pub created_at: i64,
    pub edited_at: Option<i64>,
}

impl MessageInfo {

    // Join a stored message to the files attached to it
    pub fn new(message: Message, attachments: Vec<AttachmentInfo>) -> Self {
        Self {
            seq: message.seq,
            id: message.id,
            room_id: message.room_id,
            author_id: message.author_id,
            body: message.body,
            created_at: message.created_at,
            edited_at: message.edited_at,
            attachments
        }
    }
}

// Which page of a room's history to read
pub enum Cursor {
    Latest, // Newest page
    After(i64), // Everything newer than the given seq
    Before(i64) // Page is older than the given seq
}

pub enum Posted {
    Created(Message), // Newly created
    Duplicate(Message), // Duplicate (ie connection issue)
}

pub struct Edited {
    pub room_id: Uuid,
    pub edited_at: i64,
}

// API Methods //

// Post a message to a room
pub async fn post(
    pool: &sqlx::SqlitePool,
    room_id: Uuid,
    author_id: Uuid,
    body: Option<&str>,
    client_nonce: [u8; 16],
    attachments: &[AttachmentRequest],
) -> Result<Posted> {

    // Format checks, before the transaction opens
    validate_post(body, attachments)?;

    // Start transaction
    let mut tx = pool.begin_with(db::BEGIN_IMMEDIATE).await?;

    // Check user perms
    let perms = db::effective_permissions(&mut tx, room_id, author_id).await?;
    if !can_post(perms, !attachments.is_empty()) {
        return Err(AppError::Forbidden);
    }

    // Attempt to create new message
    let message_id = Uuid::now_v7();
    let now = utils::now_ms();
    let result = sqlx::query(
        "
        INSERT INTO messages (id, room_id, author_id, body, client_nonce, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ON CONFLICT (author_id, client_nonce) DO NOTHING
        "
    )
    .bind(message_id)
    .bind(room_id)
    .bind(author_id)
    .bind(body)
    .bind(client_nonce.as_slice())
    .bind(now)
    .execute(&mut *tx)
    .await?;

    // Nothing written means this nonce is already stored (client resent)
    let posted = if result.rows_affected() == 0 {

        // Send the stored message back rather than creating a new one
        let existing = fetch_by_nonce(&mut tx, author_id, client_nonce).await?;
        Posted::Duplicate(existing)

    } else {

        attachments::link(&mut tx, message_id, attachments).await?;

        // Return newly created message as result
        Posted::Created(Message {
            seq: result.last_insert_rowid(),
            id: message_id,
            room_id,
            author_id: Some(author_id),
            body: body.map(str::to_string),
            created_at: now,
            edited_at: None
        })
    };

    // Complete transaction
    tx.commit().await?;

    Ok(posted)
}

// Read one page of a room's messages (including attachments)
pub async fn fetch(
    pool: &sqlx::SqlitePool,
    room_id: Uuid,
    user_id: Uuid,
    cursor: Cursor,
) -> Result<Vec<(Message, Vec<Attachment>)>> {

    let mut conn = pool.acquire().await?;

    // Check if user has permission to access the room
    let perms = db::effective_permissions(&mut conn, room_id, user_id).await?;
    if perms.is_none() {
        return Err(AppError::Forbidden);
    }

    // anchor is the seq to page from, older is the direction
    let (anchor, older) = match cursor {
        Cursor::Latest => (i64::MAX, true), // i64::MAX is above existing seq (returns the newest page)
        Cursor::Before(seq) => (seq, true),
        Cursor::After(seq) => (seq, false)
    };

    // Two directions from the anchor: below it descending, above it ascending
    let query = if older {
        "
        SELECT seq, id, room_id, author_id, body, created_at, edited_at
        FROM messages
        WHERE room_id = ?1 AND seq < ?2
        ORDER BY seq DESC
        LIMIT ?3
        "
    } else {
        "
        SELECT seq, id, room_id, author_id, body, created_at, edited_at
        FROM messages
        WHERE room_id = ?1 AND seq > ?2
        ORDER BY seq
        LIMIT ?3
        "
    };

    // Fetch messages and insert into vector
    let mut messages: Vec<Message> = sqlx::query_as(query)
    .bind(room_id)
    .bind(anchor)
    .bind(config::get().responses.messages_per_page)
    .fetch_all(&mut *conn)
    .await?;

    // The descending query returns newest first, callers always get ascending
    if older {
        messages.reverse();
    }

    // Empty message list, exit
    if messages.is_empty() {
        return Ok(Vec::new());
    }

    // Get attachments for all messages on page
    let low = messages[0].seq;
    let high = messages[messages.len() - 1].seq;
    let mut pairs = attachments::in_room_range(&mut conn, room_id, low, high).await?;
    let mut result = Vec::with_capacity(messages.len());
    for message in messages {
        let files = pairs.remove(&message.id).unwrap_or_default();
        result.push((message, files));
    }

    Ok(result)
}

// Read one message (with its attachments)
pub async fn get(
    pool: &sqlx::SqlitePool,
    message_id: Uuid,
    user_id: Uuid,
) -> Result<(Message, Vec<Attachment>)> {

    let mut conn = pool.acquire().await?;
    let message = fetch_by_id(&mut conn, message_id).await?;

    // Check if user has permission to access the room
    let perms = db::effective_permissions(&mut conn, message.room_id, user_id).await?;
    if perms.is_none() {
        return Err(AppError::Forbidden);
    }

    let files = attachments::in_message(pool, message.id).await?;

    Ok((message, files))
}

// Delete a message, returning the room it was in
pub async fn delete(
    pool: &sqlx::SqlitePool,
    message_id: Uuid,
    caller_id: Uuid,
) -> Result<Uuid> {

    let mut tx = pool.begin_with(db::BEGIN_IMMEDIATE).await?;
    let message = fetch_by_id(&mut tx, message_id).await?;
    let room_id = message.room_id;

    // If caller has permission to delete
    let perms = db::effective_permissions(&mut tx, room_id, caller_id).await?;
    let permitted = can_delete(perms, caller_id, message.author_id);
    let staff = db::staff_over_room(&mut tx, room_id, caller_id).await?;

    if !(permitted || staff) {
        return Err(AppError::Forbidden);
    }

    // Delete message (attachments cascade)
    sqlx::query("DELETE FROM messages WHERE id = ?1")
    .bind(message_id)
    .execute(&mut *tx)
    .await?;

    // Finish Transaction
    tx.commit().await?;

    Ok(room_id)
}

// Replace a message body, returning the room it is in and when it was edited
pub async fn edit(
    pool: &sqlx::SqlitePool,
    message_id: Uuid,
    caller_id: Uuid,
    body: Option<&str>,
) -> Result<Edited> {

    if let Some(text) = body {
        validate::message_body(text)?;
    }

    let mut tx = pool.begin_with(db::BEGIN_IMMEDIATE).await?;

    // Check author
    let message = fetch_by_id(&mut tx, message_id).await?;
    let room_id = message.room_id;

    // Does caller have permission to edit the message
    if message.author_id != Some(caller_id) {
        return Err(AppError::Forbidden);
    }

    // If there is no body, verify attachments are present
    if body.is_none() {

        // Check if original message has attachments
        let has_attachments: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM message_attachments WHERE message_id = ?1)"
        )
        .bind(message_id)
        .fetch_one(&mut *tx)
        .await?;

        if !has_attachments {
            let err = "edited message body cannot be empty and have no attachments".to_string();
            return Err(AppError::Validation(err));
        }
    }

    // Update message
    let now = utils::now_ms();
    sqlx::query(
        "
        UPDATE messages
        SET body = ?1, edited_at = ?2
        WHERE id = ?3
        "
    )
    .bind(body)
    .bind(now)
    .bind(message_id)
    .execute(&mut *tx)
    .await?;

    // Commit transaction
    tx.commit().await?;

    Ok(Edited { room_id, edited_at: now })
}

// Helper Methods //

// Check a post's body and attachment count (against limits)
fn validate_post(body: Option<&str>, attachments: &[AttachmentRequest]) -> Result<()> {

    // No body or attachments
    if body.is_none() && attachments.is_empty() {
        return Err(AppError::Validation("No message body or attachments provided".to_string()));
    }

    // Over allowed attachment limit
    let max_attachments = config::get().uploads.attachments_per_message_max;
    if attachments.len() > max_attachments {
        let err = format!("too many attachments, max allowed: {max_attachments}");
        return Err(AppError::Validation(err));
    }

    // Validate message body before using
    if let Some(text) = body {
        validate::message_body(text)?;
    }

    // Check for duplicate file IDs
    for (i, first) in attachments.iter().enumerate() {
        for second in &attachments[i + 1..] {
            if first.file_id == second.file_id {
                return Err(AppError::Validation("same file attached twice".to_string()));
            }
        }
    }

    Ok(())
}

// Read back the message a nonce already stored
async fn fetch_by_nonce(
    conn: &mut sqlx::SqliteConnection,
    author_id: Uuid,
    client_nonce: [u8; 16],
) -> Result<Message> {

    let message = sqlx::query_as::<_, Message>(
        "
        SELECT seq, id, room_id, author_id, body, created_at, edited_at
        FROM messages
        WHERE author_id = ?1 AND client_nonce = ?2
        "
    )
    .bind(author_id)
    .bind(client_nonce.as_slice())
    .fetch_one(&mut *conn)
    .await?;

    Ok(message)
}

// Reads a message by id
async fn fetch_by_id(
    conn: &mut sqlx::SqliteConnection,
    message_id: Uuid,
) -> Result<Message> {

    let message: Option<Message> = sqlx::query_as(
        "
        SELECT seq, id, room_id, author_id, body, created_at, edited_at
        FROM messages
        WHERE id = ?1
        "
    )
    .bind(message_id)
    .fetch_optional(&mut *conn)
    .await?;

    let Some(message) = message else {
        return Err(AppError::NotFound);
    };

    Ok(message)
}

// Report whether a user may post the message they sent
fn can_post(perms: Option<Vec<Permission>>, has_attachments: bool) -> bool {

    // Not a member, so the room cannot be read either
    let Some(perms) = perms else {
        return false;
    };

    // Check if they can post text messages
    if !perms.contains(&Permission::Post) {
        return false;
    }

    // Check if they can post attachments (if any are there)
    if has_attachments && !perms.contains(&Permission::Attach) {
        return false;
    }

    true
}

// Reports whether a user may delete a message
fn can_delete(perms: Option<Vec<Permission>>, caller_id: Uuid, author_id: Option<Uuid>) -> bool {

    // Not a member, so the room cannot be read
    let Some(perms) = perms else {
        return false;
    };

    // Authors can always delete their own messages
    if author_id == Some(caller_id) {
        return true;
    }

    // Deleting someone else's message takes an explicit permission
    perms.contains(&Permission::DeleteMsg)
}
