//! The `[sockets]` section of the config file.

use serde::{Deserialize, Serialize};

use crate::types::bytesize::{ByteSize, KIBIBYTE};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Sockets {
    /// Events queued for one connected user before the oldest is dropped
    pub event_queue_size: usize,

    pub participants_max: usize,

    /// Time a dropped user holds their place in the call
    pub reconnect_grace_seconds: u64,

    /// Largest connection setup message
    pub signal_size_max: ByteSize,

    /// Setup messages per participant per second
    pub signals_per_second: u32,

    /// Messages queued per voice socket before it is dropped
    pub signal_queue_size: usize,

    /// Public STUN address, as stun:host:port
    pub stun_url: Option<String>
}

impl Sockets {

    /// Errors on the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if self.event_queue_size < 1 {
            return Err("sockets.event_queue_size must be at least 1".to_string());
        }

        if self.participants_max < 2 {
            return Err("sockets.participants_max must be at least 2".to_string());
        }

        const SIGNAL_SIZE_MIN: ByteSize = ByteSize::from_int(8 * KIBIBYTE);
        if self.signal_size_max < SIGNAL_SIZE_MIN {
            return Err(format!("sockets.signal_size_max must be at least {SIGNAL_SIZE_MIN}"));
        }

        if self.signals_per_second < 1 {
            return Err("sockets.signals_per_second must be at least 1".to_string());
        }

        if self.signal_queue_size < 1 {
            return Err("sockets.signal_queue_size must be at least 1".to_string());
        }

        if let Some(url) = &self.stun_url {
            if !url.starts_with("stun:") {
                return Err("sockets.stun_url must start with stun:".to_string());
            }
        }

        Ok(())
    }
}

impl Default for Sockets {
    fn default() -> Self {
        Sockets {
            event_queue_size: 32,
            participants_max: 10,
            reconnect_grace_seconds: 30,
            signal_size_max: ByteSize::from_int(16 * KIBIBYTE),
            signals_per_second: 50,
            signal_queue_size: 256,
            stun_url: None
        }
    }
}
