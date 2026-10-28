//! A user's preferences

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/preferences.ts"))]
pub struct Folder {
    pub name: String,
    pub rooms: Vec<Uuid>,
    pub collapsed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/preferences.ts"))]
pub enum RoomListItem {
    Room(Uuid),
    Folder(Folder),
}
