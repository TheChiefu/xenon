//! The `[logging]` section of the config file.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Logging {
    /// File every tracing call is appended to, empty for stdout only
    pub file: String,

    /// Lowest level this server's own tracing calls are written at
    pub level: String
}

impl Logging {

    /// Errors on the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if !matches!(self.level.as_str(), "trace" | "debug" | "info" | "warn" | "error") {
            return Err(format!(
                "logging.level must be trace, debug, info, warn, or error, got \"{}\"",
                self.level
            ));
        }

        Ok(())
    }
}

impl Default for Logging {
    fn default() -> Self {
        Logging {
            file: "xenon.log".to_string(),
            level: "info".to_string()
        }
    }
}
