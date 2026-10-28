//! How a user appears to others

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/status.ts", repr(enum = name)))]
#[serde(rename_all = "lowercase")]
pub enum Device {
    Windows,
    Macos,
    Linux,
    Android,
    Ios,
    Desktop,
    Mobile,
    Tablet
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/status.ts", repr(enum = name)))]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum Appearance {
    Online,
    Busy,
    Away,
    Offline,
}

#[derive(Serialize, Clone)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/status.ts"))]
pub struct UserState {
    pub user_id: Uuid,
    pub appearance: Appearance,
    pub device: Option<Device>
}
