//! The `[sockets]` section of the config file.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Sockets {
    /// Events queued for one connected user before the oldest is dropped
    pub event_queue_size: usize
}

impl Sockets {

    /// Errors on the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if self.event_queue_size < 1 {
            return Err("sockets.event_queue_size must be at least 1".to_string());
        }

        Ok(())
    }
}

impl Default for Sockets {
    fn default() -> Self {
        Sockets {
            event_queue_size: 32
        }
    }
}
