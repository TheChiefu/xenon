use serde::{Deserialize, Serialize};

use crate::bytesize::{ByteSize, KIBIBYTE};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Calls {
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

impl Calls {

    /// Returns the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if self.participants_max < 2 {
            return Err("calls.participants_max must be at least 2".to_string());
        }

        if ByteSize::to_int(self.signal_size_max) < 8 * KIBIBYTE {
            return Err("calls.signal_size_max must be at least 8KiB".to_string());
        }

        if self.signals_per_second < 1 {
            return Err("calls.signals_per_second must be at least 1".to_string());
        }

        if self.signal_queue_size < 1 {
            return Err("calls.signal_queue_size must be at least 1".to_string());
        }

        if let Some(url) = &self.stun_url {
            if !url.starts_with("stun:") {
                return Err("calls.stun_url must start with stun:".to_string());
            }
        }

        Ok(())
    }
}

impl Default for Calls {
    fn default() -> Self {
        Calls {
            participants_max: 10,
            reconnect_grace_seconds: 30,
            signal_size_max: ByteSize::from_int(16 * KIBIBYTE),
            signals_per_second: 50,
            signal_queue_size: 256,
            stun_url: None
        }
    }
}
