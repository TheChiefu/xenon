//! The `[services]` section of the config file.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Services {
    /// Seconds an outgoing request to an outside service waits for a response
    pub request_timeout_seconds: u64
}

impl Default for Services {
    fn default() -> Self {
        Services {
            request_timeout_seconds: 30
        }
    }
}
