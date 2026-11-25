//! The media a user sends the server

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

/// What a user sends the server, each in a slot of its own
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "rtc/events.ts", repr(enum = name)))]
#[serde(rename_all = "snake_case")]
pub enum Media {
    Microphone,
    ScreenAudio,
    ScreenVideo
}

impl Media {
    pub const ALL: [Media; 3] = [
        Media::Microphone,
        Media::ScreenAudio,
        Media::ScreenVideo
    ];
}
