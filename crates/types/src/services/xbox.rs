//! Xbox account linking

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

/// Authorization request URL sent to client
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "services/xbox.ts"))]
pub struct LinkResponse {
    pub authorize_url: String
}

/// Caller's linked Xbox account
/// - Gamertag: Name shown for the account
/// - Honor cloaked: Whether activity is hidden while appearing offline on Xbox
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "services/xbox.ts"))]
pub struct XboxResponse {
    pub gamertag: String,
    pub honor_cloaked: bool
}

/// Settings for caller's linked account
/// - Honor cloaked: Whether activity is hidden while appearing offline on Xbox
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "services/xbox.ts"))]
pub struct XboxPatch {
    pub honor_cloaked: bool
}
