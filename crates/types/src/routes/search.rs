//! Search

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

// RESPONSES //

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/search.ts"))]
pub struct RespondSearchResult {
    pub seq: i64,
    pub id: Uuid,
    pub room_id: Uuid,
    pub author_id: Option<Uuid>,
    pub body: Option<String>,
    pub created_at: i64,
    pub edited_at: Option<i64>,
    pub files: Vec<String>, // Attached file names matching the file filter
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/search.ts"))]
pub struct RespondSearch {
    pub results: Vec<RespondSearchResult>,
    pub total: i64,
}
