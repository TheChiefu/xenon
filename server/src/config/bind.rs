//! The `[bind]` section of the config file.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Bind {
    pub ip: String,
    pub port: u16,
    /// PEM certificate chain served to clients
    pub certificate: Option<String>,

    /// PEM private key for `certificate`
    pub key: Option<String>,

    /// Origins allowed to make cross-origin requests, e.g. a web client
    /// served from a different address. Empty allows none.
    pub allowed_origins: Vec<String>,

    // Max age for a CORS request
    pub cors_max_age: u64,

    /// Connections have 'X-Forwarded-For' header read as the client IP
    pub trusted_proxies: Vec<IpAddr>
}

impl Bind {

    /// Returns the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if self.port == 0 {
            return Err("bind.port must be greater than 0".to_string());
        }

        if self.ip.parse::<IpAddr>().is_err() {
            return Err(format!("bind.ip must be an IPv4 or IPv6 address, got \"{}\"", self.ip));
        }

        match (&self.certificate, &self.key) {
            (Some(_), None) => return Err("bind.key must be set alongside bind.certificate".to_string()),
            (None, Some(_)) => return Err("bind.certificate must be set alongside bind.key".to_string()),
            _ => {}
        }

        Ok(())
    }
}

impl Default for Bind {
    fn default() -> Self {
        Bind {
            ip: "127.0.0.1".to_string(),
            port: 3000,
            certificate: None,
            key: None,
            allowed_origins: Vec::new(),
            cors_max_age: 3600,

            // Default to localhost as trusted proxy source
            trusted_proxies: vec![
                IpAddr::V4(Ipv4Addr::LOCALHOST),
                IpAddr::V6(Ipv6Addr::LOCALHOST)
            ],
        }
    }
}
