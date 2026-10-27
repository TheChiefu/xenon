//! What the server pushes to a connected client, and what one may send back.

pub mod call;
pub mod platforms;
pub mod presence;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::shared::{Platform, Status};
use crate::routes::messages::MessageResponse;
use crate::sockets::chat::presence::Device;
use crate::sockets::events::call::ActiveCall;
use crate::sockets::events::platforms::{Presence, UserActivity};
use crate::sockets::events::presence::UserPresence;

/// What a client sends over its socket
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "sockets/events.ts"))]
pub enum ClientEvent {
    /// The status this connection declares
    Status { status: Status }
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
        room_id: Uuid
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
        users: Vec<UserActivity>
    },
    GamePresenceSnapshot {
        users: Vec<UserActivity>
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
        platform: Platform,
        presence: Option<Presence>
    },
    CallSnapshot {
        calls: Vec<ActiveCall>
    },
    ParticipantJoined {
        room_id: Uuid,
        user_id: Uuid
    },
    ParticipantLeft {
        room_id: Uuid,
        user_id: Uuid
    },
    SessionRevoked
}
