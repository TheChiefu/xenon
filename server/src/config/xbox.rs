//! The `[xbox]` section of the config file

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Xbox {
    /// Entra app registration accepting personal accounts
    pub client_id: String,

    /// Entra client secret value
    pub client_secret: String,

    /// Address registered on that app, which Microsoft redirects a browser to
    pub redirect_uri: String,

    /// Path to a JSON file renaming games and their rich presence
    pub game_mappings: String,

    /// Seconds between presence polls
    pub poll_interval_seconds: u64
}

impl Xbox {

    /// Returns the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if self.client_id.is_empty() {
            return Err("xbox.client_id must be set".to_string());
        }

        if self.client_secret.is_empty() {
            return Err("xbox.client_secret must be set".to_string());
        }

        if !self.redirect_uri.starts_with("http://") && !self.redirect_uri.starts_with("https://") {
            return Err("xbox.redirect_uri must start with http:// or https://".to_string());
        }

        if self.poll_interval_seconds < 1 {
            return Err("xbox.poll_interval_seconds must be at least 1".to_string());
        }

        Ok(())
    }
}

impl Default for Xbox {
    fn default() -> Self {
        Xbox {
            client_id: String::new(),
            client_secret: String::new(),
            redirect_uri: String::new(),
            game_mappings: String::new(),
            poll_interval_seconds: 60
        }
    }
}
