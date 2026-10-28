//! The `[rate_limit]` section of the config file.

use serde::{Deserialize, Serialize};

/// How often requests are accepted
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct RateLimit {
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

impl RateLimit {

    /// Errors on the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if self.address_register_failures_max < 1 {
            return Err("rate_limit.address_register_failures_max must be at least 1".to_string());
        }

        if self.account_login_failures_max < 1 {
            return Err("rate_limit.account_login_failures_max must be at least 1".to_string());
        }

        if self.address_login_failures_max < 1 {
            return Err("rate_limit.address_login_failures_max must be at least 1".to_string());
        }

        Ok(())
    }
}

impl Default for RateLimit {
    fn default() -> Self {
        RateLimit {
            address_register_failures_max: 5,
            address_login_failures_max: 20,
            account_login_failures_max: 10,
            block_on_register_limit: true,
            block_on_login_limit: true
        }
    }
}
