//! The `[push]` section of the config file.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Push {
    /// Where a push service reaches the operator, a mailto: or https:// address
    pub subject: String,

    /// Seconds a push service stores a message for a device that is switched off
    pub ttl: u32
}

impl Push {

    /// Errors on the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if !self.subject.starts_with("mailto:") && !self.subject.starts_with("https://") {
            return Err("push.subject must start with mailto: or https://".to_string());
        }

        Ok(())
    }
}

impl Default for Push {
    fn default() -> Self {
        Push {
            subject: String::new(),
            ttl: 86400
        }
    }
}
