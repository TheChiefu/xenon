//! The `[services]` section of the config file.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Services {
    /// Seconds an outgoing request to an outside service waits for a response
    pub request_timeout_seconds: u64
}

impl Services {

    /// Errors on the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if self.request_timeout_seconds < 1 {
            return Err("services.request_timeout_seconds must be at least 1".to_string());
        }

        Ok(())
    }
}

impl Default for Services {
    fn default() -> Self {
        Services {
            request_timeout_seconds: 30
        }
    }
}
