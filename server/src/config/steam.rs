//! The `[steam]` section of the config file

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Steam {
    pub web_api_key: String, // Steam Web API key used to read player summaries
    pub poll_interval_seconds: u64, // Seconds between presence polls
}

impl Steam {

    /// Errors on the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if self.web_api_key.is_empty() {
            return Err("steam.web_api_key must be set".to_string());
        }

        if self.poll_interval_seconds < 1 {
            return Err("steam.poll_interval_seconds must be at least 1".to_string());
        }

        Ok(())
    }
}

impl Default for Steam {
    fn default() -> Self {
        Steam {
            web_api_key: String::new(),
            poll_interval_seconds: 60
        }
    }
}
