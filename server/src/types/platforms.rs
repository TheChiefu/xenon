//! Platform types for reporting

use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/platforms.ts", repr(enum = name)))]
pub enum Availability {
    Online,
    Offline,
    Away
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/platforms.ts"))]
pub struct Metadata {
    pub availability: Availability,
    pub game: Option<String>,
    pub activity: Option<String>
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/platforms.ts"))]
pub struct Activity {
    pub xbox: Option<Metadata>,
    pub steam: Option<Metadata>
}

impl Activity {
    pub fn get(&self, platform: Platform) -> Option<&Metadata> {
        match platform {
            Platform::Xbox => self.xbox.as_ref(),
            Platform::Steam => self.steam.as_ref()
        }
    }

    pub fn set(&mut self, platform: Platform, metadata: Option<Metadata>) {
        match platform {
            Platform::Xbox => self.xbox = metadata,
            Platform::Steam => self.steam = metadata
        }
    }
}

#[derive(Serialize, Clone)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/platforms.ts"))]
pub struct UserActivity {
    pub user_id: Uuid,
    pub activity: Activity
}
