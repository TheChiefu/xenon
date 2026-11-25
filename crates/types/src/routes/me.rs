//! The caller's own account

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

// REQUESTS //

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/me.ts"))]
pub struct RequestPasswordSet {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/me.ts"))]
pub struct RequestDeleteAccount {
    /// Deletes every message the account wrote
    #[serde(default)]
    pub delete_history: bool,
}

// RESPONSES //

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/me.ts"))]
pub struct RespondRoomReadState {
    pub room_id: Uuid,
    pub last_read_seq: i64,
    pub unread: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/me.ts"))]
pub struct RespondRoomInvite {
    pub room_id: Uuid,
    pub room_name: Option<String>
}

// PATCHES //

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/me.ts"))]
pub struct PatchUserProfile {
    #[serde(default)]
    pub display_name: Option<String>,

    /// Empty string clears the text
    #[serde(default)]
    pub description: Option<String>,

    /// Nil UUID clears the avatar
    #[serde(default)]
    pub avatar_file_id: Option<Uuid>,

    /// Nil UUID clears the banner
    #[serde(default)]
    pub banner_file_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/me.ts"))]
pub struct PatchRoomPreferences {
    #[serde(default)]
    pub room_layout: Option<String>,
}
