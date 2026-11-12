//! Platform types for reporting

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/platforms.ts", repr(enum = name)))]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum Platform {
    Xbox,
    Steam,
}
