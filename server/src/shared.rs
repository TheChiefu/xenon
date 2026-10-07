//! Types shared between the database and the client.
//!
//! enums marked with "PERMANENT" is stored as its integer. Never reuse or
//! renumber a retired variant's number once rows exist.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

// Roles //

/// A user's server-wide role
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "shared.ts"))]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum GlobalRole {
    Owner,
    Admin,
    Member,
    Visitor,
}

/// How a room is discovered and entered
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "shared.ts"))]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum Visibility {
    /// Self service
    Public,
    /// Invite only
    Locked,
    /// Invite only
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "shared.ts"))]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum Notify {
    None,
    Mentions,
    All,
}

/// A game service an account is linked to
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "shared.ts"))]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum Platform {
    Xbox,
    Steam,
}

/// How a user appears to others
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "shared.ts"))]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum Status {
    Online,
    Busy,
    Away,
    Offline,
}

// Permissions //

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "shared.ts"))]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    /// Send messages
    Post,
    /// Attach files to a message
    Attach,
    /// Use slash commands (unimplemented)
    Commands,
    /// Delete other users' messages
    DeleteMsg,
    /// Create invites to the room
    Invite,
    /// Edit the room name and visibility
    Rename,
    /// Remove a user from the room, with or without an expiry
    Ban,
    /// Set other users' permissions, bounded by your own
    Grant,
    /// Delete the room
    DeleteRoom,
    /// Join voice chat (unimplemented)
    Connect,
    /// Speak in voice chat (unimplemented)
    Speak,
    /// Mute others in voice chat (unimplemented)
    Mute,
    /// Show webcam video (unimplemented)
    Video,
    /// Share screen (unimplemented)
    Screenshare,
}

impl Permission {
    /// Every permission - Add new ones here
    pub const ALL: [Permission; 14] = [
        Self::Post,
        Self::Attach,
        Self::Commands,
        Self::DeleteMsg,
        Self::Invite,
        Self::Rename,
        Self::Ban,
        Self::Grant,
        Self::DeleteRoom,
        Self::Connect,
        Self::Speak,
        Self::Mute,
        Self::Video,
        Self::Screenshare,
    ];

    /// Reports whether the permission is aimed at a member, and so cannot be
    /// used against someone who also holds it
    #[must_use]
    pub const fn member_directed(self) -> bool {
        matches!(self, Self::Ban | Self::Grant | Self::Mute)
    }
}

/// A `linked_accounts` row.
#[derive(sqlx::FromRow, Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "shared.ts"))]
pub struct LinkedAccount {
    // Platform's name (ie. Xbox)
    pub platform: Platform,

    /// Name shown for the account
    pub handle: String,
}

/// A named group of rooms in a user's room list layout
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "shared.ts"))]
pub struct Folder {
    pub name: String,
    pub rooms: Vec<Uuid>,
    pub collapsed: bool,
}

/// One entry in a user's room list layout, room ID and possible folder it's in
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "shared.ts"))]
pub enum RoomListItem {
    Room(Uuid),
    Folder(Folder),
}
