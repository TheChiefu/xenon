use serde::Serialize;
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::shared::Status;
use crate::sockets::chat::presence::Device;

/// A user and what the reader is told to show for them
#[derive(Serialize, Clone)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "sockets/events.ts"))]
pub struct UserPresence {
    pub user_id: Uuid,
    pub status: Status,

    /// Unset until the user's client declares one
    pub device: Option<Device>
}
