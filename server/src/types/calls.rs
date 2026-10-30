//! Who is in each room's call, as sent to clients

use std::collections::HashSet;

use serde::Serialize;
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

#[derive(Serialize, Clone)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/calls.ts"))]
pub struct RoomCall {
    pub room_id: Uuid,
    pub participants: HashSet<Uuid>
}
