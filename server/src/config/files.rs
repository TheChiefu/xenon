//! The `[files]` section of the config file.

use serde::{Deserialize, Serialize};

use crate::types::bytesize::{ByteSize, MEBIBYTE};
use crate::types::time::{DAY, SECOND};

/// Where uploaded files are kept, and how large they may be
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Files {
    /// Directory holding uploaded files, sharded by hash
    pub path: String,

    /// Largest single uploaded file
    pub file_size_max: ByteSize,

    /// Most files attached to one message
    pub attachments_per_message_max: usize,

    /// Seconds a browser may cache a downloaded file
    pub cache_max_age: u64
}

impl Files {

    /// Errors on the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if self.path.is_empty() {
            return Err("files.path must be set".to_string());
        }

        if self.file_size_max.to_int() <= 0 {
            return Err("files.file_size_max must be at least 1 byte".to_string());
        }

        Ok(())
    }
}

impl Default for Files {
    fn default() -> Self {
        Files {
            path: "files".to_string(),
            file_size_max: ByteSize::from_int(25 * MEBIBYTE),
            attachments_per_message_max: 10,
            cache_max_age: (7 * DAY / SECOND) as u64
        }
    }
}
