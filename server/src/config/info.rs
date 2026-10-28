//! The `[info]` section of the config file.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Info {
    pub name: String,
    pub kind: String,
    pub description: String,
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
