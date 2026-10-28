//! Propeties of a user

use core::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::error::{AppError, Result};
use crate::types::platforms::Platform;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/users.ts", repr(enum = name)))]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum GlobalRole {
    Admin,
    Member,
    Visitor,
}

impl fmt::Display for GlobalRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GlobalRole::Admin => write!(f, "Admin"),
            GlobalRole::Member => write!(f, "Member"),
            GlobalRole::Visitor => write!(f, "Visitor"),
        }
    }
}

impl FromStr for GlobalRole {
    type Err = AppError;

    fn from_str(role: &str) -> Result<Self> {
        match role {
            "admin" => Ok(GlobalRole::Admin),
            "member" => Ok(GlobalRole::Member),
            "visitor" => Ok(GlobalRole::Visitor),
            _ => Err(AppError::Validation(format!("unknown role \"{role}\""))),
        }
    }
}

#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/users.ts"))]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub description: String,
    pub avatar_file_id: Option<Uuid>,
    pub banner_file_id: Option<Uuid>,
    pub global_role: GlobalRole,
    pub created_at: i64,
    pub banned: bool,
    pub linked_accounts: Vec<LinkedAccount>,
}

#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/users.ts"))]
pub struct LinkedAccount {
    pub platform: Platform,
    pub handle: String,
}
