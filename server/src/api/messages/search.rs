//! Message search

use std::collections::HashMap;

use uuid::Uuid;

use crate::api::messages::Message;
use crate::config;
use crate::error::Result;
use crate::shared::SearchRequest;
use crate::validate;

// Data Structs //

/// A matched message with the names of its matching files
pub struct Found {
    pub message: Message,
    pub files: Vec<String>,
}

// API Methods //

/// Searches messages in every room the user is a member of
pub async fn search(
    pool: &sqlx::SqlitePool,
    user_id: Uuid,
    request: &SearchRequest,
) -> Result<Vec<Found>> {

    validate::search(request)?;

    let mut conn = pool.acquire().await?;

    // Build query
    let mut builder = sqlx::QueryBuilder::new(
        "SELECT m.seq, m.id, m.room_id, m.author_id, m.body, m.created_at, m.edited_at "
    );
    push_source(&mut builder, user_id, request);
    push_filters(&mut builder, request);

    // Page newest first
    builder.push(" ORDER BY m.seq DESC LIMIT ");
    builder.push_bind(config::get().limits.search_page);

    // Fetch page
    let messages: Vec<Message> = builder.build_query_as().fetch_all(&mut *conn).await?;

    // Matching file names for the page
    let mut names = match &request.file {
        Some(file) => matching_files(&mut conn, &messages, file).await?,
        None => HashMap::new(),
    };

    // Pair messages with their file names
    let mut found = Vec::with_capacity(messages.len());
    for message in messages {
        let files = names.remove(&message.id).unwrap_or_default();
        found.push(Found { message, files });
    }

    Ok(found)
}

// Helper Methods //

/// Builds the FROM clause from the request
fn push_source(builder: &mut sqlx::QueryBuilder<sqlx::Sqlite>, user_id: Uuid, request: &SearchRequest) {

    // Text starts from its index and other filters from messages
    if request.text.is_some() {
        builder.push("FROM messages_fts JOIN messages m ON m.seq = messages_fts.rowid ");
    } else {
        builder.push("FROM messages m ");
    }

    // Only rooms with access
    builder.push("JOIN room_access a ON a.room_id = m.room_id AND a.user_id = ");
    builder.push_bind(user_id);
}

/// Builds the WHERE clause from the request
fn push_filters(builder: &mut sqlx::QueryBuilder<sqlx::Sqlite>, request: &SearchRequest) {

    // No cursor starts at the newest match
    builder.push(" WHERE m.seq < ");
    builder.push_bind(request.before.unwrap_or(i64::MAX));

    // Message body
    if let Some(text) = &request.text {
        builder.push(" AND messages_fts MATCH ");
        builder.push_bind(phrase(text));
    }

    // Attached file names
    if let Some(file) = &request.file {
        builder.push(
            " AND m.id IN (
                SELECT ma.message_id
                FROM files_fts
                JOIN files f ON f.seq = files_fts.rowid
                JOIN message_attachments ma ON ma.file_id = f.id
                WHERE files_fts MATCH "
        );
        builder.push_bind(phrase(file));
        builder.push(")");
    }

    // Room name
    if let Some(room) = &request.room {
        builder.push(" AND m.room_id IN (SELECT id FROM rooms WHERE name = ");
        builder.push_bind(room.clone());
        builder.push(")");
    }

    // Author username
    if let Some(author) = &request.author {
        builder.push(" AND m.author_id IN (SELECT id FROM users WHERE username = ");
        builder.push_bind(author.clone());
        builder.push(")");
    }

    // Date range start
    if let Some(from) = request.from {
        builder.push(" AND m.created_at >= ");
        builder.push_bind(from);
    }

    // Date range end
    if let Some(to) = request.to {
        builder.push(" AND m.created_at < ");
        builder.push_bind(to);
    }
}

/// Finds matching file names per message
async fn matching_files(
    conn: &mut sqlx::SqliteConnection,
    messages: &[Message],
    file: &str,
) -> Result<HashMap<Uuid, Vec<String>>> {

    let mut names: HashMap<Uuid, Vec<String>> = HashMap::new();
    if messages.is_empty() {
        return Ok(names);
    }

    let mut builder = sqlx::QueryBuilder::new(
        "
        SELECT ma.message_id, f.filename
        FROM files_fts
        JOIN files f ON f.seq = files_fts.rowid
        JOIN message_attachments ma ON ma.file_id = f.id
        WHERE files_fts MATCH "
    );
    builder.push_bind(phrase(file));
    builder.push(" AND ma.message_id IN (");

    // Each push adds one placeholder and holds its value
    let mut list = builder.separated(", ");
    for message in messages {
        list.push_bind(message.id);
    }
    list.push_unseparated(") ORDER BY ma.ordinal");

    // Group file names by message
    let rows: Vec<(Uuid, String)> = builder.build_query_as().fetch_all(&mut *conn).await?;
    for (message_id, filename) in rows {
        names.entry(message_id).or_default().push(filename);
    }

    Ok(names)
}

/// Quotes search text so FTS5 reads it as plain text
fn phrase(text: &str) -> String {
    format!("\"{}\"", text.replace('"', "\"\""))
}
