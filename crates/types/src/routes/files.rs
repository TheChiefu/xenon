//! File upload and download

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/files.ts"))]
pub struct RespondFile {
    pub id: Uuid,
    pub filename: String,
    pub mime: String,
    pub byte_size: i64,
}
