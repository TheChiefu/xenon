//! Message types used in requests and responses

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/messages.ts"))]
pub struct AttachmentRequest {
    pub file_id: Uuid,
    #[serde(default)]
    pub spoiler: bool,
}

/// POST body for a message search
#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/messages.ts"))]
pub struct SearchRequest {
    pub text: Option<String>,
    pub file: Option<String>,
    pub room: Option<String>, // Exact search
    pub author: Option<String>, // Exact search
    pub from: Option<i64>,
    pub to: Option<i64>, // Exclusive
    pub page: i64, // Starts at 1
}

#[derive(Clone, Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/messages.ts"))]
pub struct AttachmentInfo {
    pub id: Uuid,
    pub filename: String,
    pub mime: String,
    pub byte_size: i64,
    pub spoiler: bool,
}

/// A posted message's notification and who receives it
pub struct Notification {
    pub room_id: Uuid,
    pub room_name: String,
    pub author: String,
    pub body: String,
    pub recipients: Vec<Uuid>,
}

#[derive(Clone, Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/messages.ts"))]
pub struct MessageInfo {
    pub seq: i64,
    pub id: Uuid,
    pub room_id: Uuid,
    pub author_id: Option<Uuid>,
    pub body: Option<String>,
    pub created_at: i64,
    pub edited_at: Option<i64>,
    pub attachments: Vec<AttachmentInfo>
}
