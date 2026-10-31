//! HTTP handlers reporting what this server is.

use axum::Json;
use serde::Serialize;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::config;

// DATA STRUCTS //

/// General server information and limits clients need to know about
#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/server.ts"))]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub kind: String,
    pub description: String,

    /// Limits clients must stay within
    pub limit_users_lookup: usize,
    pub limit_message_page: i64,
    pub limit_search_page: i64,
    pub limit_room_page: i64,
    pub limit_users_page: i64,
    pub limit_file_bytes_max: i64,
    pub limit_attachments_per_message_max: usize,
}

// ROUTING METHODS //

/// Get the server's information
pub async fn info() -> Json<ServerInfo> {

    let config = config::get();

    let info = ServerInfo {
        name: config.server.name.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        kind: config.server.kind.clone(),
        description: config.server.description.clone(),
        limit_users_lookup: config.responses.users_per_lookup,
        limit_message_page: config.responses.messages_per_page,
        limit_search_page: config.responses.search_results_per_page,
        limit_room_page: config.responses.rooms_per_page,
        limit_users_page: config.responses.users_per_page,
        limit_file_bytes_max: config.files.file_size_max.to_int(),
        limit_attachments_per_message_max: config.files.attachments_per_message_max,
    };

    Json(info)
}

/// Get the server version
pub async fn version() -> Json<String> {
    let version = env!("CARGO_PKG_VERSION").to_string();
    Json(version)
}

/// Get what kind of server is deployed, such as "Development" or "Release"
pub async fn kind() -> Json<String> {
    let kind = config::get().server.kind.clone();
    Json(kind)
}
