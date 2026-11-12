//! HTTP handlers reporting what this server is.

use axum::Json;
use types::routes::server::ServerInfo;

use crate::config;

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
        #[cfg(feature = "rtc")]
        stun_url: config.sockets.stun_url.clone(),
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
