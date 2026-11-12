//! Room properties

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(rename_all = "lowercase"))]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/rooms.ts", repr(enum = name)))]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Public, // Discoverable
    Hidden, // Invite only
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(rename_all = "lowercase"))]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/rooms.ts", repr(enum = name)))]
#[serde(rename_all = "lowercase")]
pub enum Notify {
    None,
    Mentions,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/rooms.ts", repr(enum = name)))]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    Post, // Send messages
    Attach, // Attach files to a message
    DeleteMsg, // Delete other users' messages
    Invite, // Create invites to the rooms
    Rename, // Edit the room name and visibility
    Ban, // Remove a user from the room
    Grant, // Set other users' permissions
    DeleteRoom, // Delete the room
    Mute, // Mute others
    Kick, // Kick others in call
}

impl Permission {
    /// Every permission
    pub const ALL: [Permission; 10] = [
        Self::Post,
        Self::Attach,
        Self::DeleteMsg,
        Self::Invite,
        Self::Rename,
        Self::Ban,
        Self::Grant,
        Self::DeleteRoom,
        Self::Mute,
        Self::Kick,
    ];

}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/rooms.ts"))]
pub struct RoomInfo {
    pub id: Uuid,
    pub name: String,
    pub visibility: Visibility,
    pub default_permissions: Vec<Permission>,
}
