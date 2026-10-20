//! The `[rate_limit]` section of the config file.

use serde::{Deserialize, Serialize};

/// How often requests are accepted
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct RateLimit {
    /// Registration attempts one IP can make per window
    pub register_attempts_max: u32,

    /// Minutes before an IP's registration attempts reset
    pub register_window_minutes: u64,

    /// Failed logins in a row before an account is locked
    pub login_failures_max: u32
}

impl RateLimit {

    /// Returns the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if self.register_attempts_max < 1 {
            return Err("rate_limit.register_attempts_max must be at least 1".to_string());
        }

        if self.register_window_minutes < 1 {
            return Err("rate_limit.register_window_minutes must be at least 1".to_string());
        }

        if self.login_failures_max < 1 {
            return Err("rate_limit.login_failures_max must be at least 1".to_string());
        }

        Ok(())
    }
}

impl Default for RateLimit {
    fn default() -> Self {
        RateLimit {
            register_attempts_max: 5,
            register_window_minutes: 60,
            login_failures_max: 10
        }
    }
}
