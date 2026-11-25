//! The `[server]` section of the config file.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Server {
    pub name: String,
    pub kind: String,
    pub description: String,

    /// File every tracing call is appended to, empty for stdout only
    pub log_file: String,

    /// Lowest level this server's own tracing calls are written at
    pub log_level: String
}

impl Server {

    /// Errors on the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if !matches!(self.log_level.as_str(), "trace" | "debug" | "info" | "warn" | "error") {
            return Err(format!(
                "server.log_level must be trace, debug, info, warn, or error, got \"{}\"",
                self.log_level
            ));
        }

        Ok(())
    }
}

impl Default for Server {
    fn default() -> Self {
        Server {
            name: "Xenon Server".to_string(),
            kind: "Development".to_string(),
            description: "My custom Xenon server".to_string(),
            log_file: "xenon.log".to_string(),
            log_level: "info".to_string()
        }
    }
}
