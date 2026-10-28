//! HTTP handlers for file upload and download.

use axum::body::Body;
use axum::extract::{Multipart, Path, State};
use axum::http::{header, HeaderMap, HeaderName, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use futures_util::TryStreamExt;
use serde::Serialize;
use sqlx::SqlitePool;
use tokio_util::io::{ReaderStream, StreamReader};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use uuid::Uuid;

use crate::api::files::Stored;
use crate::types::bytesize::MEBIBYTE;
use crate::error::{AppError, Result};
use crate::api::files::File;
use crate::routes::AuthUser;
use crate::api;
use crate::config;
use crate::validate;

// Data Structs //

#[derive(Clone, Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/files.ts"))]
pub struct RespondFile {
    pub id: Uuid,
    pub filename: String,
    pub mime: String,
    pub byte_size: i64,
}

impl From<File> for RespondFile {
    fn from(file: File) -> Self {
        Self {
            id: file.id,
            filename: file.filename,
            mime: file.mime,
            byte_size: file.byte_size,
        }
    }
}

// Header carrying a downloaded file's MIME type
pub const FILE_MIME_HEADER: HeaderName = HeaderName::from_static("x-file-mime");

// Routing Methods //

// Store an uploaded file
pub async fn upload(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<RespondFile>)> {

    // Reject unallowed roles to upload
    api::files::require_upload(&pool, caller_id).await?;

    // Reject a request with no content length
    let Some(value) = headers.get(header::CONTENT_LENGTH) else {
        return Err(AppError::Validation("upload must declare a content length".to_string()));
    };

    // Reject a content length that is not visible ASCII
    let Ok(text) = value.to_str() else {
        return Err(AppError::Validation("upload content length is unreadable".to_string()));
    };

    // Reject a content length that is not a number
    let Ok(bytes) = text.parse::<usize>() else {
        return Err(AppError::Validation("upload content length is not a number".to_string()));
    };

    // Reject a body over the body limit before reading it
    if bytes > max_body_bytes() {
        return Err(AppError::TooLarge(config::get().uploads.file_size_max));
    }

    // Read POST body
    let field = match multipart.next_field().await {
        Ok(Some(field)) => field,
        Ok(None) => {
            let err = "the file upload request had no multipart fields";
            return Err(AppError::Validation(err.to_string()));
        }
        Err(e) => return Err(AppError::Io(std::io::Error::other(e)))
    };

    // Get file name
    let file_path: &str = field.file_name()
        .ok_or(AppError::Validation("no file name provided on file upload".to_string()))?;

    // Setup reader
    let file_name = validate::file_name(file_path)?;
    let reader = StreamReader::new(field.map_err(std::io::Error::other));

    // Get result and return outcome
    let result = api::files::store(&pool, &file_name, reader).await?;
    match result {
        Stored::Created(file) => Ok((StatusCode::CREATED, Json(file.into()))),
        Stored::Duplicate(file) => Ok((StatusCode::OK, Json(file.into())))
    }
}

// Stream a stored file
pub async fn download(
    AuthUser(..): AuthUser,
    State(pool): State<SqlitePool>,
    Path(file_id): Path<Uuid>,
) -> Result<Response> {

    // Destructure the result
    let (file, handle) = api::files::fetch(&pool, file_id).await?;

    // Wrap handle into body and provide headers
    let body = Body::from_stream(ReaderStream::new(handle));
    let headers = [
        (header::CONTENT_TYPE, "application/octet-stream".to_string()),
        (header::CONTENT_DISPOSITION, "attachment".to_string()),
        (header::CONTENT_LENGTH, file.byte_size.to_string()),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string()),
        (header::CACHE_CONTROL, "private, max-age=31536000, immutable".to_string()),
        (FILE_MIME_HEADER, file.mime.clone())
    ];

    // Send response
    Ok((StatusCode::OK, headers, body).into_response())
}

// Return the largest upload body accepted
pub fn max_body_bytes() -> usize {
    let headroom_bytes = MEBIBYTE;
    let max_file_bytes = config::get().uploads.file_size_max.to_int();
    (headroom_bytes + max_file_bytes) as usize
}
