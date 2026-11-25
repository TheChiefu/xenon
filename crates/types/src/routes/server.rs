//! What a server is

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

/// General server information and limits clients need to know about
#[derive(Debug, Serialize, Deserialize)]
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

    /// STUN server for calls, when one is set
    #[cfg(feature = "rtc")]
    pub stun_url: Option<String>,
}
