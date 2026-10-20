//! The `[info]` section of the config file.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Info {
    pub name: String,
    pub kind: String,
    pub description: String,
}

impl Info {

    /// Warns about every empty value in the section
    pub(super) fn validate(&self) {

        if self.name.is_empty() {
            tracing::warn!("no server name provided")
        }

        if self.kind.is_empty() {
            tracing::warn!("no server kind/type provided")
        }

        if self.description.is_empty() {
            tracing::warn!("no server description provided")
        }
    }
}

impl Default for Info {
    fn default() -> Self {
        Info {
            name: "Xenon Server".to_string(),
            kind: "Development".to_string(),
            description: "My custom Xenon server".to_string(),
        }
    }
}
