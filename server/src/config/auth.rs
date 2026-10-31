//! The `[auth]` section of the config file.

use serde::{Deserialize, Serialize};

/// How sessions last, and how often failed attempts are accepted
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Auth {
    /// How long a session survives without activity
    pub lifetime_days: i64,

    /// Days that must pass before an active session's expiry is extended
    /// again, limiting how often it is written
    pub renew_after_days_elapsed: i64,

    pub registration_code_required: bool,

    /// Failed registrations one IP can make
    pub address_register_failures_max: u32,

    /// Failed logins one IP can make across all accounts
    pub address_login_failures_max: u32,

    /// Failed logins in a row before an account is locked
    pub account_login_failures_max: u32,

    /// Block an IP that uses all of its registration attempts
    pub block_on_register_limit: bool,

    /// Block an IP that uses all of its failed logins
    pub block_on_login_limit: bool
}

impl Auth {

    /// Errors on the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if self.lifetime_days < 1 {
            return Err("auth.lifetime_days must be at least 1".to_string());
        }

        // Equal values leave no renewal window, so sessions never extend
        if self.renew_after_days_elapsed >= self.lifetime_days {
            return Err("auth.renew_after_days_elapsed must be less than auth.lifetime_days".to_string());
        }

        if self.address_register_failures_max < 1 {
            return Err("auth.address_register_failures_max must be at least 1".to_string());
        }

        if self.account_login_failures_max < 1 {
            return Err("auth.account_login_failures_max must be at least 1".to_string());
        }

        if self.address_login_failures_max < 1 {
            return Err("auth.address_login_failures_max must be at least 1".to_string());
        }

        Ok(())
    }
}

impl Default for Auth {
    fn default() -> Self {
        Auth {
            lifetime_days: 30,
            renew_after_days_elapsed: 1,
            registration_code_required: true,
            address_register_failures_max: 5,
            address_login_failures_max: 20,
            account_login_failures_max: 10,
            block_on_register_limit: true,
            block_on_login_limit: true
        }
    }
}
