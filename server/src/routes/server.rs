//! HTTP handlers reporting what this server is.

use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::config;
use crate::error::Result;

// Data Structs //

/// General server information and limits clients need to know about
#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/server.ts"))]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub kind: String,
    pub description: String,

    /// Limits sent to clients to
    pub limit_users_lookup: usize,
    pub limit_message_page: i64,
    pub limit_search_page: i64,
    pub limit_room_page: i64,
    pub limit_users_page: i64,
    pub limit_file_bytes_max: i64,
    pub limit_attachments_per_message_max: usize,
}

// Routing Methods //

/// Get the server's information
pub async fn info() -> Result<(StatusCode, Json<ServerInfo>)> {

    let info = ServerInfo {
        name: config::get().info.name.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        kind: config::get().info.kind.clone(),
        description: config::get().info.description.clone(),
        limit_users_lookup: config::get().responses.users_per_lookup,
        limit_message_page: config::get().responses.messages_per_page,
        limit_search_page: config::get().responses.search_results_per_page,
        limit_room_page: config::get().responses.rooms_per_page,
        limit_users_page: config::get().responses.users_per_page,
        limit_file_bytes_max: config::get().uploads.file_size_max.to_int(),
        limit_attachments_per_message_max: config::get().uploads.attachments_per_message_max,
    };

    Ok((StatusCode::OK, Json(info)))
}

/// Get the server version
pub async fn version() -> Result<Json<String>> {
    let version = env!("CARGO_PKG_VERSION").to_string();
    Ok(Json(version))
}

/// Get what kind of server is deployed, such as "Development" or "Release"
pub async fn kind() -> Result<Json<String>> {
    let kind = config::get().info.kind.clone();
    Ok(Json(kind))
}

/// Respond that the server is up and healthy
pub async fn health_check() -> Result<(StatusCode, Json<String>)> {
    Ok((StatusCode::OK, Json("Up and running".to_string())))
}