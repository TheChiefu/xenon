//! The `message_attachments` table: which files hang off which message.

use std::collections::HashMap;

use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::api::files::File;
use crate::types::messages::{AttachmentInfo, AttachmentRequest};

// Data Structs //

#[derive(sqlx::FromRow)]
pub struct Attachment {
    #[sqlx(flatten)]
    pub file: File,
    pub spoiler: bool,
}

impl From<Attachment> for AttachmentInfo {
    fn from(attachment: Attachment) -> Self {
        Self {
            id: attachment.file.id,
            filename: attachment.file.filename,
            mime: attachment.file.mime,
            byte_size: attachment.file.byte_size,
            spoiler: attachment.spoiler,
        }
    }
}

#[derive(sqlx::FromRow)]
struct AttachmentRow {
    message_id: Uuid,
    #[sqlx(flatten)]
    attachment: Attachment,
}

// API Methods //

// Link file(s) to a message
pub async fn link(
    conn: &mut sqlx::SqliteConnection,
    message_id: Uuid,
    attachments: &[AttachmentRequest],
) -> Result<()> {

    let sql = "
        INSERT INTO message_attachments (message_id, file_id, ordinal, spoiler)
        VALUES (?1, ?2, ?3, ?4)
    ";

    // Iterate over each attachment and insert into DB
    for (pos, attachment) in attachments.iter().enumerate() {
        let result = sqlx::query(sql)
        .bind(message_id)
        .bind(attachment.file_id)
        .bind(pos as i64)
        .bind(attachment.spoiler)
        .execute(&mut *conn)
        .await;

        if let Err(e) = result {

            // No "files" row for that id (client named a file that is gone)
            if let sqlx::Error::Database(db) = &e {
                if db.is_foreign_key_violation() {
                    let id = attachment.file_id;
                    return Err(AppError::Validation(format!("no file with id {id}")));
                }
            }

            return Err(AppError::Db(e));
        }
    }

    Ok(())
}

// Read every file attached to a message
pub async fn in_message(
    pool: &sqlx::SqlitePool,
    message_id: Uuid,
) -> Result<Vec<Attachment>> {

    let result = sqlx::query_as::<_, Attachment>(
        "
        SELECT f.id, f.sha256, f.filename, f.mime, f.byte_size, ma.spoiler
        FROM message_attachments ma
        JOIN files f ON f.id = ma.file_id
        WHERE ma.message_id = ?1
        ORDER BY ma.ordinal
        "
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?;

    Ok(result)
}

// Read all attachments of a page of messages
pub async fn in_room_range(
    conn: &mut sqlx::SqliteConnection,
    room_id: Uuid,
    start: i64,
    end: i64,
) -> Result<HashMap<Uuid, Vec<Attachment>>> {

    let rows = sqlx::query_as::<_, AttachmentRow>(
        "
        SELECT ma.message_id, f.id, f.sha256, f.filename, f.mime, f.byte_size, ma.spoiler
        FROM message_attachments ma
        JOIN messages m ON m.id = ma.message_id
        JOIN files f ON f.id = ma.file_id
        WHERE m.room_id = ?1 AND m.seq BETWEEN ?2 AND ?3
        ORDER BY ma.message_id, ma.ordinal
        "
    )
    .bind(room_id)
    .bind(start)
    .bind(end)
    .fetch_all(&mut *conn)
    .await?;

    let mut files: HashMap<Uuid, Vec<Attachment>> = HashMap::new();
    for row in rows {
        files.entry(row.message_id).or_default().push(row.attachment);
    }

    Ok(files)
}
