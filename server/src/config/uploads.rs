//! The `[uploads]` section of the config file.

use serde::{Deserialize, Serialize};

use crate::types::bytesize::{ByteSize, MEBIBYTE};

/// Size of uploaded files and attachments
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Uploads {
    /// Largest single uploaded file
    pub file_size_max: ByteSize,

    /// Most files attached to one message
    pub attachments_per_message_max: usize
}

impl Uploads {

    /// Errors on the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if self.file_size_max.to_int() <= 0 {
            return Err("uploads.file_size_max must be at least 1 byte".to_string());
        }

        Ok(())
    }
}

impl Default for Uploads {
    fn default() -> Self {
        Uploads {
            file_size_max: ByteSize::from_int(25 * MEBIBYTE),
            attachments_per_message_max: 10
        }
    }
}
