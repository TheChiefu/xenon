//! The `files` table, and the bytes it points at on disk

use std::path::{Path, PathBuf};

use futures_util::{stream, StreamExt, TryStreamExt};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

use crate::types::bytesize::ByteSize;
use crate::config;
use crate::db;
use crate::error::{AppError, Result};
use crate::types::users::GlobalRole;
use crate::utils;

// Data Structs //

// A `files` row
#[derive(sqlx::FromRow)]
pub struct File {
    pub id: Uuid,
    pub sha256: Vec<u8>,
    pub filename: String,
    pub mime: String,
    pub byte_size: i64,
}

// Outcome of an upload
pub enum Stored {
    Created(File), // New file
    Duplicate(File) // File already exists
}

// Metadata produced by reading an upload
struct Stream {
    sha256: Vec<u8>,
    byte_size: i64,
    mime: &'static str
}

// API Methods //

// Check whether a user's role may upload files
pub async fn can_upload(
    pool: &SqlitePool,
    caller_id: Uuid,
) -> Result<bool> {
    let mut conn = pool.acquire().await?;
    let role = db::global_role(&mut conn, caller_id).await?;
    Ok(role == GlobalRole::Admin || role == GlobalRole::Member)
}

/// Store an upload, keyed on the hash of its bytes
pub async fn store<R>(
    pool: &SqlitePool,
    filename: &str,
    reader: R,
) -> Result<Stored>
where
    R: AsyncRead + Unpin,
{
    let files_path = &config::get().storage.files;

    // End file location is unknown until hashed,
    // write with tmp location/name until finished
    let tmp_path = Path::new(files_path)
        .join("tmp")
        .join(Uuid::now_v7().to_string());

    // Read stream into tmp file
    let max_bytes = config::get().uploads.file_size_max;
    let stream = match read_stream(reader, &tmp_path, max_bytes).await {
        Ok(val) => val,
        Err(e) => {
            // On any error, remove tmp file
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(e);
        }
    };

    // Build sharded path and move tmp file into it (permanent location)
    let hex = hex::encode(&stream.sha256);
    let shard_dir = create_shard_path(files_path, &hex);
    if let Err(e) = move_to_shard(&tmp_path, &shard_dir, &hex).await {
        // Move failed, delete tmp file to avoid leaving unused data on disk
        let _ = tokio::fs::remove_file(&tmp_path).await;
        return Err(e);
    }
    // NOTE: In the event of an orphaned file a sweeping event that looks for
    // them (ie. no references in the DB) can delete them at a later point

    // Insert file info into Database
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let stored = insert(&mut tx, filename, stream.sha256, stream.byte_size, stream.mime).await?;
    tx.commit().await?;

    Ok(stored)
}

// Read a file's row and open a handle to its bytes on disk
pub async fn fetch(
    pool: &SqlitePool,
    file_id: Uuid,
) -> Result<(File, tokio::fs::File)> {

    // Select file from table by id
    let mut conn = pool.acquire().await?;
    let f = sqlx::query_as::<_, File>(
        "
        SELECT id, sha256, filename, mime, byte_size
        FROM files
        WHERE id = ?1
        "
    )
    .bind(file_id)
    .fetch_optional(&mut *conn)
    .await?;

    // No file found, report as not found error
    let Some(file) = f else {
        return Err(AppError::NotFound);
    };

    // Rebuild path based on SHA256
    let hex_sha = hex::encode(&file.sha256);
    let directory = create_shard_path(&config::get().storage.files, &hex_sha);
    let path = directory.join(&hex_sha);

    // Get file reader handle
    match tokio::fs::File::open(&path).await {
        Ok(handle) => Ok((file, handle)),
        Err(e) => {
            // Expected a "files" row, but has no bytes on disk (storage inconsistency)
            if e.kind() == std::io::ErrorKind::NotFound {
                tracing::error!("file {file_id} has no bytes at {}", path.display());
            }
            Err(AppError::Io(e))
        }
    }
}

// Deletes rows nothing references, returning the deleted rows
pub async fn delete_unused_rows(pool: &SqlitePool) -> Result<Vec<File>> {
    let files = sqlx::query_as::<_, File>(
        "
        DELETE FROM files
        WHERE id NOT IN (SELECT file_id FROM message_attachments)
          AND id NOT IN (SELECT file_id FROM user_files)
          AND id NOT IN (SELECT avatar_file_id FROM users WHERE avatar_file_id IS NOT NULL)
          AND id NOT IN (SELECT banner_file_id FROM users WHERE banner_file_id IS NOT NULL)
        RETURNING id, sha256, filename, mime, byte_size
        "
    )
    .fetch_all(pool)
    .await?;

    Ok(files)
}

// Deletes each file's bytes from disk
pub async fn delete_unused_files(files: &[File], threads: usize) -> Result<()> {
    let files_path = &config::get().storage.files;

    let mut deletes = Vec::new();
    let mut bytes_deleted = 0;
    for file in files {
        let hex = hex::encode(&file.sha256);
        let path = create_shard_path(files_path, &hex).join(&hex);
        deletes.push(tokio::fs::remove_file(path));
        bytes_deleted += file.byte_size;
    }

    stream::iter(deletes)
        .buffer_unordered(threads)
        .try_collect::<Vec<()>>()
        .await?;

    println!(
        "Deleted {0} unused files ({1} bytes)",
        files.len(),
        bytes_deleted,
    );

    Ok(())
}

// Helper Methods //

/// Reports whether a chunk of bytes reads as UTF-8 text
fn is_utf8_text(chunk: &[u8]) -> bool {

    // Check for NULs
    if chunk.contains(&0) {
        return false;
    }

    match std::str::from_utf8(chunk) {
        Ok(_) => true,
        Err(e) => e.error_len().is_none()
    }
}


// Maps a hash to the directory holding it, such as `files/ab/cd`
fn create_shard_path(files_path: &str, hex: &str) -> PathBuf {
    Path::new(files_path)
        .join(&hex[0..2]) // Top Level first 2 hex chars
        .join(&hex[2..4]) // Second Level second 2 hex chars
}

// Create shard directory and rename the temp file into it
async fn move_to_shard(tmp_path: &Path, shard_dir: &Path, hex: &str) -> Result<()> {
    tokio::fs::create_dir_all(shard_dir).await?;
    tokio::fs::rename(tmp_path, shard_dir.join(hex)).await?;
    Ok(())
}

// Reads a stream into a temp file, returning the metadata it gathered
async fn read_stream<R>(
    mut reader: R,
    tmp_path: &Path,
    max_bytes: ByteSize,
) -> Result<Stream>
where
    R: AsyncRead + Unpin,
{
    let mut tmp_file = tokio::fs::File::create(tmp_path).await?;

    // Setup reader and metadata properties
    let mut buffer = [0u8; 65536]; // 64KiB chunks
    let mut read_bytes: i64 = 0;
    let mut hasher = Sha256::new();
    let mut mime = "application/octet-stream";
    let max = max_bytes.to_int();

    // Loop over file bytes
    loop {
        let n = reader.read(&mut buffer).await?; // Read bytes (dynamic size)
        if n == 0 {
            break; // No more bytes, stop reading
        }
        let chunk = &buffer[..n];

        // Infer matches leading bytes as file MIME
        if read_bytes == 0 {
            if let Some(val) = infer::get(chunk) {
                mime = val.mime_type();
            } else if is_utf8_text(chunk) {
                mime = "text/plain";
            }
        }

        // Update read bytes
        read_bytes += n as i64;

        // File reached limit (disallow)
        if read_bytes > max {
            return Err(AppError::TooLarge(max_bytes));
        }

        // Update tmp file and hasher by chunks
        tmp_file.write_all(chunk).await?;
        hasher.update(chunk);
    }

    // Reject empty files
    if read_bytes == 0 {
        return Err(AppError::Validation("file is empty".to_string()));
    }

    // Finish/flush buffered writes to OS
    // (in case a failure needs be returned)
    tmp_file.flush().await?;

    // Return read file metadata and hash
    Ok(Stream {
        sha256: hasher.finalize().to_vec(),
        byte_size: read_bytes,
        mime
    })
}

// Writes a row to the `files` table, keyed on the hash of its bytes
async fn insert(
    conn: &mut sqlx::SqliteConnection,
    filename: &str,
    sha256: Vec<u8>,
    byte_size: i64,
    mime: &'static str,
) -> Result<Stored> {

    let id = Uuid::now_v7();
    let now = utils::now_ms();

    // Attempt to insert file, if already exists do nothing
    let affected = sqlx::query(
        "
        INSERT INTO files (id, sha256, filename, mime, byte_size, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ON CONFLICT (sha256) DO NOTHING
        "
    )
    .bind(id)
    .bind(sha256.as_slice())
    .bind(filename)
    .bind(mime)
    .bind(byte_size)
    .bind(now)
    .execute(&mut *conn)
    .await?
    .rows_affected();

    // Nothing written means these bytes are already stored
    if affected == 0 {

        let file = sqlx::query_as::<_, File>(
            "
            SELECT id, sha256, filename, mime, byte_size
            FROM files
            WHERE sha256 = ?1
            "
        )
        .bind(sha256.as_slice())
        .fetch_one(&mut *conn)
        .await?;

        return Ok(Stored::Duplicate(file));
    }

    // If an insert occurred, return newly created file info
    Ok(Stored::Created(
        File {
            id,
            sha256,
            filename: filename.to_string(),
            mime: mime.to_string(),
            byte_size
        }
    ))
}