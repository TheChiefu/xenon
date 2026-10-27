use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::Platform;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

// A linked account's status on its platform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "sockets/events.ts", repr(enum = name)))]
pub enum Status {
    Online,
    Offline,
    Away
}

/// What a linked account last reported about the user who owns it
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "sockets/events.ts"))]
pub struct Presence {
    pub status: Status,
    pub game: Option<String>,
    pub activity: Option<String>
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "sockets/events.ts"))]
pub struct Activity {
    pub xbox: Option<Presence>,
    pub steam: Option<Presence>
}

impl Activity {
    pub fn get(&self, platform: Platform) -> Option<&Presence> {
        match platform {
            Platform::Xbox => self.xbox.as_ref(),
            Platform::Steam => self.steam.as_ref()
        }
    }

    pub fn set(&mut self, platform: Platform, presence: Option<Presence>) {
        match platform {
            Platform::Xbox => self.xbox = presence,
            Platform::Steam => self.steam = presence
        }
    }
}

/// A user and what each of their linked accounts last reported
#[derive(Serialize, Clone)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "sockets/events.ts"))]
pub struct UserActivity {
    pub user_id: Uuid,
    pub activity: Activity
}
