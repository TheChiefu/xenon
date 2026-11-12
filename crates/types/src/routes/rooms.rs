//! Rooms and the tables scoped to a single room

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::rooms::{Notify, Permission, Visibility};

// REQUESTS //

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/rooms.ts"))]
pub struct RequestCreateRoom {
    pub name: String,
    pub visibility: Visibility,
    pub default_permissions: Vec<Permission>,
    pub claim_all: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/rooms.ts"))]
pub struct RequestRead {
    pub last_read_seq: i64,
}

// RESPONSES //

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/rooms.ts"))]
pub struct RespondCreated {
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/rooms.ts"))]
pub struct RespondDiscoverability {
    pub id: Uuid,
    pub name: String,
    pub visibility: Visibility,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/rooms.ts"))]
pub struct RespondMember {
    pub user_id: Uuid,
    pub permissions: Vec<Permission>,
    pub granted_at: i64,
    pub notify: Option<Notify>
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/rooms.ts"))]
pub struct RespondAccess {
    pub permissions: Option<Vec<Permission>>,
    pub invited: bool,
    pub banned: bool,
}

// PATCHES //

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/rooms.ts"))]
pub struct Patch {
    pub name: Option<String>,
    pub visibility: Option<Visibility>,
    pub default_permissions: Option<Vec<Permission>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/rooms.ts"))]
pub struct PatchAccess {
    pub permissions: Option<Vec<Permission>>,
    pub notify: Option<Notify>
}

// DATA STRUCTS //

/// Query string for a paged room listing
#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryQuery {
    pub after: Option<Uuid>,
    pub limit: Option<i64>
}
