//! The `[storage]` section of the config file.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Storage {
    /// Directory holding uploaded files, sharded by hash
    pub files: String
}

impl Storage {

    /// Returns the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if self.files.is_empty() {
            return Err("storage.files must be set".to_string());
        }

        Ok(())
    }
}

impl Default for Storage {
    fn default() -> Self {
        Storage {
            files: "files".to_string()
        }
    }
}
