//! The `[network]` section of the config file.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use serde::{Deserialize, Serialize};
use url::Url;

use crate::types::time::{HOUR, SECOND};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Network {
    pub ip: String,
    pub port: u16,
    /// PEM certificate chain served to clients
    pub certificate: Option<String>,

    /// PEM private key for the certificate
    pub key: Option<String>,

    /// Origins allowed to make cross-origin requests, e.g. a web client
    /// served from a different address. Empty allows none.
    pub allowed_origins: Vec<String>,

    /// Seconds a browser may cache a CORS preflight response
    pub cors_max_age: u64,

    /// Connections have 'X-Forwarded-For' header read as the client IP
    pub trusted_proxies: Vec<IpAddr>,

    pub public_url: Option<String>,

    /// Seconds an outgoing request to an outside service waits for a response
    pub request_timeout_seconds: u64
}

impl Network {

    /// Errors on the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if self.port == 0 {
            return Err("network.port must be greater than 0".to_string());
        }

        if self.ip.parse::<IpAddr>().is_err() {
            return Err(format!("network.ip must be an IPv4 or IPv6 address, got \"{}\"", self.ip));
        }

        match (&self.certificate, &self.key) {
            (Some(_), None) => return Err("network.key must be set alongside network.certificate".to_string()),
            (None, Some(_)) => return Err("network.certificate must be set alongside network.key".to_string()),
            _ => {}
        }

        if let Some(url) = &self.public_url {
            match Url::parse(url) {
                Ok(parsed) if matches!(parsed.scheme(), "http" | "https") => {}
                _ => return Err(format!("network.public_url must be a valid http:// or https:// URL, got \"{url}\"")),
            }
        }

        if self.request_timeout_seconds < 1 {
            return Err("network.request_timeout_seconds must be at least 1".to_string());
        }

        Ok(())
    }
}

impl Default for Network {
    fn default() -> Self {
        Network {
            ip: "127.0.0.1".to_string(),
            port: 3000,
            certificate: None,
            key: None,
            allowed_origins: Vec::new(),
            cors_max_age: (1 * HOUR / SECOND) as u64,

            // Default to localhost as trusted proxy source
            trusted_proxies: vec![
                IpAddr::V4(Ipv4Addr::LOCALHOST),
                IpAddr::V6(Ipv6Addr::LOCALHOST)
            ],
            public_url: None,
            request_timeout_seconds: 30
        }
    }
}
