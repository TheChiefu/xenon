//! The `[sockets]` section of the config file.

#[cfg(feature = "rtc")]
use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Sockets {
    /// Events queued for one connected user before the oldest is dropped
    pub event_queue_size: usize,

    #[cfg(feature = "rtc")]
    pub participants_max: usize,

    /// UDP ports RTC connections use, forwarded on the router
    #[cfg(feature = "rtc")]
    pub udp_ports: RangeInclusive<u16>,

    /// STUN server for calls
    #[cfg(feature = "rtc")]
    pub stun_url: Option<String>
}

impl Sockets {

    /// Errors on the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if self.event_queue_size < 1 {
            return Err("sockets.event_queue_size must be at least 1".to_string());
        }

        #[cfg(feature = "rtc")]
        {
            if self.participants_max < 2 {
                return Err("sockets.participants_max must be at least 2".to_string());
            }

            if *self.udp_ports.start() == 0 {
                return Err("sockets.udp_ports.start must be greater than 0".to_string());
            }

            if self.udp_ports.is_empty() {
                return Err("sockets.udp_ports.end must be at least sockets.udp_ports.start".to_string());
            }

            // https://www.rfc-editor.org/rfc/rfc7064#section-3.1
            if let Some(stun_url) = &self.stun_url
                && !stun_url.starts_with("stun:")
                && !stun_url.starts_with("stuns:")
            {
                return Err("sockets.stun_url must start with stun: or stuns:".to_string());
            }
        }

        Ok(())
    }
}

impl Default for Sockets {
    fn default() -> Self {
        Sockets {
            event_queue_size: 32,
            #[cfg(feature = "rtc")]
            participants_max: 10,
            #[cfg(feature = "rtc")]
            udp_ports: 50000..=50099,
            #[cfg(feature = "rtc")]
            stun_url: None
        }
    }
}
