//! Types shared between the database and the client.
//!
//! enums marked with "PERMANENT" is stored as its integer. Never reuse or
//! renumber a retired variant's number once rows exist.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

// Roles //

/// A user's server-wide role, stored as an integer. (PERMANENT)
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "shared.ts"))]
#[serde(rename_all = "lowercase")]
#[repr(i8)]
pub enum GlobalRole {
    Owner = 0,
    Admin = 1,
    Member = 2,
    Visitor = 3,
}

/// How a room is discovered and entered, stored as an integer. (PERMANENT)
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "shared.ts"))]
#[serde(rename_all = "lowercase")]
#[repr(i8)]
pub enum Visibility {
    /// Self service
    Public = 0,
    /// Invite only
    Locked = 1,
    /// Invite only
    Hidden = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[repr(i8)]
pub enum Notify {
    None = 0,
    Mentions = 1,
    All = 2,
}

/// A game service an account is linked to, stored as an integer. (PERMANENT)
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "shared.ts"))]
#[serde(rename_all = "lowercase")]
#[repr(i8)]
pub enum Platform {
    Xbox = 0,
    Steam = 1,
}

/// What a user asks to appear as while connected, stored as an integer. (PERMANENT)
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "shared.ts"))]
#[serde(rename_all = "lowercase")]
#[repr(i8)]
pub enum Status {
    Online = 0,
    Busy = 1,
    Away = 2,
    Invisible = 3,
}

// Permissions //

/// One bit position in a [`Permissions`] mask. (PERMANENT)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "shared.ts"))]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum Permission {
    /// Send messages
    Post = 0,
    /// Attach files to a message
    Attach = 1,
    /// Use slash commands (unimplemented)
    Commands = 2,
    /// Delete other users' messages
    DeleteMsg = 3,
    /// Create invites to the room
    Invite = 4,
    /// Edit the room name and visibility
    Rename = 5,
    /// Remove a user from the room, with or without an expiry
    Ban = 6,
    /// Set other users' permissions, bounded by your own
    Grant = 7,
    /// Delete the room
    DeleteRoom = 8,
    /// Join voice chat (unimplemented)
    Connect = 9,
    /// Speak in voice chat (unimplemented)
    Speak = 10,
    /// Mute others in voice chat (unimplemented)
    Mute = 11,
    /// Show webcam video (unimplemented)
    Video = 12,
    /// Share screen (unimplemented)
    Screenshare = 13,
}

impl Permission {
    /// Reports whether the permission is aimed at a member, and so cannot be
    /// used against someone who also holds it
    #[must_use]
    pub const fn member_directed(self) -> bool {
        matches!(self, Self::Ban | Self::Grant | Self::Mute)
    }
}

/// A set of [`Permission`] bits, stored as an integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct Permissions(i64);

impl Permissions {
    pub const NONE: Self = Self(0);

    /// Every permission - Add new ones here
    pub const FULL: Self = Self::NONE
        .grant(Permission::Post)
        .grant(Permission::Attach)
        .grant(Permission::Commands)
        .grant(Permission::DeleteMsg)
        .grant(Permission::Invite)
        .grant(Permission::Rename)
        .grant(Permission::Ban)
        .grant(Permission::Grant)
        .grant(Permission::DeleteRoom)
        .grant(Permission::Connect)
        .grant(Permission::Speak)
        .grant(Permission::Mute)
        .grant(Permission::Video)
        .grant(Permission::Screenshare);

    /// Reports whether the mask holds the given permission.
    ///
    /// # Arguments
    ///
    /// * `p` - Permission to test for.
    #[must_use]
    pub fn has(self, p: Permission) -> bool {
        let perm = p as u8;
        let bit = 1i64 << perm; // Shift by 'perm' bits left
        self.0 & bit != 0 // AND | Check if 'perm' bit is set (0 - No, 1 -Yes)
    }

    /// Returns the mask with the given permission added.
    ///
    /// # Arguments
    ///
    /// * `p` - Permission to add.
    #[must_use]
    pub const fn grant(self, p: Permission) -> Self {
        let perm = p as u8;
        let bit = 1i64 << perm;
        Self(self.0 | bit) // Turn 'perm' bit ON
    }

    /// Returns the mask with the given permission removed.
    ///
    /// # Arguments
    ///
    /// * `p` - Permission to remove.
    #[must_use]
    pub fn revoke(self, p: Permission) -> Self {
        let perm = p as u8;
        let bit = 1i64 << perm;
        Self(self.0 & !bit) // Turn 'perm' bit OFF
    }

    /// Reports whether every permission in the given set is also in this one.
    ///
    /// # Arguments
    ///
    /// * `p` - Permission set that must be covered.
    pub fn contains(self, p: Permissions) -> bool {
        p.0 & !self.0 == 0 // No bit of 'p' is absent from self
    }

    /// Builds a mask from a list of permissions.
    ///
    /// # Arguments
    ///
    /// * `perms` - Permissions the mask holds.
    pub fn from_list(perms: &[Permission]) -> Permissions {
        let mut output = Self::NONE;
        for perm in perms {
            output = output.grant(*perm);
        }
        output
    }
}

impl Default for Permissions {
    fn default() -> Self {
        Self::NONE
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

/// One entry in a user's room list layout
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "shared.ts"))]
pub enum RoomListItem {
    /// A bare room id
    Room(Uuid),
    Folder(Folder),
}
