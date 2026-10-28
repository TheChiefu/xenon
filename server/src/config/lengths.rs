//! The `[lengths]` section of the config file.

use serde::{Deserialize, Serialize};

/// Character counts allowed in text fields
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Lengths {
    pub username_min: usize,
    pub username_max: usize,
    pub display_name_min: usize,
    pub display_name_max: usize,
    pub password_min: usize,
    pub password_max: usize,
    pub profile_description_max: usize,
    pub room_name_max: usize,
    pub message_body_max: usize
}

impl Lengths {

    /// Errors on the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if self.username_min < 1 || self.username_max < self.username_min {
            return Err("lengths.username_min must be at least 1 and no greater than lengths.username_max".to_string());
        }

        if self.display_name_min < 1 || self.display_name_max < self.display_name_min {
            return Err("lengths.display_name_min must be at least 1 and no greater than lengths.display_name_max".to_string());
        }

        if self.password_min < 1 || self.password_max < self.password_min {
            return Err("lengths.password_min must be at least 1 and no greater than lengths.password_max".to_string());
        }

        if self.room_name_max < 1 {
            return Err("lengths.room_name_max must be at least 1".to_string());
        }

        if self.message_body_max < 1 {
            return Err("lengths.message_body_max must be at least 1".to_string());
        }

        Ok(())
    }
}

impl Default for Lengths {
    fn default() -> Self {
        Lengths {
            username_min: 2,
            username_max: 32,
            display_name_min: 1,
            display_name_max: 64,
            password_min: 8,
            password_max: 128,
            profile_description_max: 2000,
            room_name_max: 128,
            message_body_max: 8000
        }
    }
}
