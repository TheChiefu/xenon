//! What the server pushes to a connected client, and what one may send back.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::shared::{Platform, Status};
use crate::routes::messages::MessageResponse;
use crate::sockets::presence::Device;

/// What a client sends over its socket.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "sockets/events.ts"))]
pub enum ClientEvent {
    /// The status this connection declares
    Status { status: Status }
}

/// How a linked game account appears to someone sharing a room with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "sockets/events.ts", repr(enum = name)))]
pub enum GamePresence {
    Online,
    Offline,
    Away
}

/// What a linked account last reported about the user who owns it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "sockets/events.ts"))]
pub struct GameActivity {
    pub platform: Platform,
    pub status: GamePresence,

    /// Name of the game
    pub title: Option<String>,

    /// What the game says they are doing
    pub detail: Option<String>
}

/// A user and the linked account they are on.
#[derive(Serialize, Clone)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "sockets/events.ts"))]
pub struct UserGamePresence {
    pub user_id: Uuid,

    #[serde(flatten)]
    pub game: GameActivity
}

/// A user and what the reader is told to show for them.
#[derive(Serialize, Clone)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "sockets/events.ts"))]
pub struct UserPresence {
    pub user_id: Uuid,
    pub status: Status,

    /// Unset until the user's client declares one
    pub device: Option<Device>
}

#[derive(Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "sockets/events.ts"))]
pub enum ServerEvent {
    Message {
        room_id: Uuid,
        message: MessageResponse
    },
    MessageDeleted {
        room_id: Uuid,
        message_id: Uuid
    },
    MessageEdited {
        room_id: Uuid,
        message_id: Uuid,
        body: Option<String>,
        edited_at: i64 
    },
    Invited {
        room_id: Uuid,
        invited_by: Uuid
    },
    InviteRevoked {
        room_id: Uuid
    },
    Banned {
        room_id: Uuid
    },
    MemberJoined {
        room_id: Uuid,
        user_id: Uuid
    },
    MemberLeft {
        room_id: Uuid,
        user_id: Uuid
    },
    RoomDeleted {
        room_id: Uuid
    },
    RoomUpdated {
        room_id: Uuid
    },
    PresenceUpdated {
        user_id: Uuid,
        status: Status,
        device: Option<Device>
    },
    PresenceSnapshot {
        users: Vec<UserPresence>
    },
    GamePresenceSnapshot {
        users: Vec<UserGamePresence>
    },
    ProfileUpdated {
        user_id: Uuid,
        display_name: String,
        description: String,
        avatar_file_id: Option<Uuid>,
        banner_file_id: Option<Uuid>
    },
    Resync,
    Notification {
        room_id: Uuid,
        room_name: String,
        author: String,
        body: String
    },
    GamePresenceUpdated {
        user_id: Uuid,

        #[serde(flatten)]
        game: GameActivity
    }
}
