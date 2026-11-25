//! Users

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::users::GlobalRole;

// REQUESTS //

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/users.ts"))]
pub struct RequestRoleSet {
    pub role: GlobalRole,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/users.ts", optional_fields))]
pub struct RequestUsers {
    #[serde(default)]
    pub ids: Option<Vec<Uuid>>,

    /// Start of a username to match
    #[serde(default)]
    pub username: Option<String>,

    #[serde(default)]
    pub after: Option<Uuid>,

    #[serde(default)]
    pub limit: Option<i64>,
}

// RESPONSES //

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/users.ts"))]
pub struct RespondSummary {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_file_id: Option<Uuid>,
    pub banner_file_id: Option<Uuid>,
    pub global_role: GlobalRole,
}
