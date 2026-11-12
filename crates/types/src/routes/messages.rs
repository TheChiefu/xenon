//! Messages

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::messages::AttachmentRequest;

// REQUESTS //

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/messages.ts"))]
pub struct RequestMessageSend {
    pub body: Option<String>,
    pub client_nonce: String,
    #[serde(default)]
    pub attachments: Vec<AttachmentRequest>
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/messages.ts"))]
pub struct RequestMessageEdit {
    pub body: Option<String>
}

// RESPONSES //

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/messages.ts"))]
pub struct RespondMessageEdit {
    pub edited_at: i64
}

// DATA STRUCTS //

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchQuery {
    pub after: Option<i64>,
    pub before: Option<i64>,
}
