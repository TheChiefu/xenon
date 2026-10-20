//! The `[session]` section of the config file.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Session {
    /// How long a session survives without activity
    pub lifetime_days: i64,

    /// Days that must elapse before an active session is extended again.
    /// Activity always extends expiry to `lifetime_days` from now, and this
    /// limits how often that write happens
    pub renew_after_days_elapsed: i64,

    pub registration_code_required: bool
}

impl Session {

    /// Returns the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if self.lifetime_days < 1 {
            return Err("session.lifetime_days must be at least 1".to_string());
        }

        // Equal values leave no renewal window, so sessions never extend
        if self.renew_after_days_elapsed >= self.lifetime_days {
            return Err("session.renew_after_days_elapsed must be less than session.lifetime_days".to_string());
        }

        Ok(())
    }
}

impl Default for Session {
    fn default() -> Self {
        Session {
            lifetime_days: 30,
            renew_after_days_elapsed: 1,
            registration_code_required: true
        }
    }
}
