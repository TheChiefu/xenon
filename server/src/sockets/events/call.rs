use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

/// What a client sends over its call socket
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "sockets/events.ts"))]
pub enum CallClientEvent {
    Signal { to: Uuid, payload: String }
}

/// Codes the server closes a call socket with
#[derive(Clone, Copy)]
#[repr(u16)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "sockets/events.ts", repr(enum), rename_all = "snake_case"))]
pub enum CallCloseCode {
    /// RFC 6455's code for a server failure
    ServerError = 1011,

    // RFC 6455 leaves 4000-4999 to applications
    NoConnect = 4001,
    AlreadyInCall = 4002,
    CallFull = 4003
}

/// A room's call and who is in it
#[derive(Serialize, Clone)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "sockets/events.ts"))]
pub struct ActiveCall {
    pub room_id: Uuid,
    pub participants: Vec<Uuid>
}

/// A STUN or TURN server a browser uses to find a route to its peers
#[derive(Serialize, Clone)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "sockets/events.ts"))]
pub struct IceServer {
    pub urls: String
}

#[derive(Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "sockets/events.ts"))]
pub enum CallServerEvent {
    CallJoined {
        participants: Vec<Uuid>,
        ice_servers: Vec<IceServer>
    },
    Signal {
        from: Uuid,
        payload: String
    }
}
