//! What the server pushes to a connected client, and what one may send back.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::types::calls::RoomCall;
use crate::types::messages::MessageInfo;
use crate::types::platforms::{Metadata, Platform, UserActivity};
use crate::types::status::{Appearance, Device, UserState};

/// What a client sends over its socket
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "sockets/session/events.ts"))]
pub enum ClientEvent {
    /// The status this connection declares
    Status { status: Appearance }
}

#[derive(Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "sockets/session/events.ts"))]
pub enum ServerEvent {
    Message {
        room_id: Uuid,
        message: MessageInfo
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
    StatusUpdated {
        user_id: Uuid,
        status: Appearance,
        device: Option<Device>
    },
    StatusSnapshot {
        users: Vec<UserState>
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
    PlatformSnapshot {
        users: Vec<UserActivity>
    },
    PlatformUpdated {
        user_id: Uuid,
        platform: Platform,
        metadata: Option<Metadata>
    },
    CallSnapshot {
        calls: Vec<RoomCall>
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
