//! Notifying a room's members of a posted message.

use std::collections::HashMap;

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::api::rooms;
use crate::api::rooms::access::{self, NotifyUserPair};
use crate::api::users;
use crate::error::Result;
use crate::types::messages::Notification;
use crate::types::rooms::Notify;

// PUBLIC METHODS //

/// Notify a room's members
pub async fn notify(
    pool: &SqlitePool,
    room_id: Uuid,
    author_id: Uuid,
    body: Option<&str>,
) -> Result<Notification> {

    // Use a placeholder body for messages with only attachments
    let body = match body {
        Some(val) => val,
        None => "Attachment(s)"
    };

    // Check who is mentioned
    let mentioned = mentioned_ids(body);

    // The author is a member, so the room resolves for them
    let room = rooms::get(pool, room_id, author_id).await?;
    let author = users::get(pool, author_id).await?;

    // Get display names of everyone mentioned
    let names = users::display_names(pool, &mentioned).await?;
    let body = name_mentions(body, &names);

    // Get list of recipients eligible for receiving the notification
    let pairs = access::list_notify_pairs(pool, room_id).await?;
    let recipients = recipients(&pairs, author_id, &mentioned);

    Ok(Notification {
        room_id,
        room_name: room.name,
        author: author.display_name,
        body,
        recipients
    })
}

// HELPER METHODS //

/// Get list of IDs eligible for notification
fn recipients(
    pairs: &[NotifyUserPair],
    author_id: Uuid,
    mentioned: &[Uuid],
) -> Vec<Uuid> {
    let mut found = Vec::new();

    for pair in pairs {

        // Author does not notify themselves
        if pair.user_id == author_id {
            continue;
        }

        // Users who do not opt in don't get notified
        if pair.notify == Notify::None {
            continue;
        }

        // Users who fully opt in, get notified
        if pair.notify == Notify::All {
            found.push(pair.user_id);
            continue;
        }

        // Users who are mentioned, get notified
        if pair.notify == Notify::Mentions && mentioned.contains(&pair.user_id) {
            found.push(pair.user_id);
        }
    }

    found
}

/// Collect accounts a message body mentions
fn mentioned_ids(body: &str) -> Vec<Uuid> {
    let mut found: Vec<Uuid> = Vec::new();
    let mut text = body;

    while let Some(byte_index) = text.find('@') {

        // Retrieve and parse UUID from mention
        let after_at = byte_index + 1;
        let id = mention_id(text, after_at);

        if let Some(id) = id {
            if !found.contains(&id) {
                found.push(id);
            }
        }

        // Next iteration starts after current mention
        text = &text[after_at..];
    }

    found
}

/// Replace each mention "@UUID" with account's display name
fn name_mentions(body: &str, names: &HashMap<Uuid, String>) -> String {
    let mut output = String::with_capacity(body.len());
    let mut text = body;

    while let Some(byte_index) = text.find('@') {

        // Retrieve and parse UUID from mention
        let after_at: usize = byte_index + 1;
        let id = mention_id(text, after_at);

        // Copies text up to and including the '@'
        output.push_str(&text[..after_at]);

        // The display name is written instead of the uuid
        if let Some(id) = id {
            if let Some(name) = names.get(&id) {
                output.push_str(name);
                text = &text[after_at + uuid::fmt::Hyphenated::LENGTH..];
                continue;
            }
        }

        // Next iteration starts after current mention
        text = &text[after_at..];
    }

    output.push_str(text);
    output
}

/// Extract UUID from mention
fn mention_id(text: &str, after_at: usize) -> Option<Uuid> {
    text.get(after_at..after_at + uuid::fmt::Hyphenated::LENGTH)
        .and_then(|uuid| Uuid::try_parse(uuid).ok())
}
