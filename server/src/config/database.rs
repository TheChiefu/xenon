//! The `[database]` section of the config file.

use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Database {
    /// SQLite database file
    pub path: String,

    /// Most connections open at once
    pub max_connections: u32,

    /// Connections held open while unused
    pub min_connections: u32,

    /// Seconds a request waits for a free connection
    pub acquire_timeout_seconds: u64,

    /// Seconds an unused connection stays open, 0 to keep it open
    pub idle_timeout_seconds: u64,

    /// Seconds a statement waits for the write lock
    pub busy_timeout_seconds: u64,

    /// Times a broadcast retries reading who to send to, on top of the first try
    pub broadcast_retries: u32,

    /// Milliseconds waited between broadcast retries
    pub broadcast_retry_delay_ms: u64,
}

impl Database {

    /// Returns the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if self.path.is_empty() {
            return Err("database.path must be set".to_string());
        }

        if self.max_connections < 1 {
            return Err("database.max_connections must be at least 1".to_string());
        }

        if self.min_connections > self.max_connections {
            return Err("database.min_connections must not exceed database.max_connections".to_string());
        }

        Ok(())
    }

    /// How long an unused connection stays open, or `None` to keep it open.
    pub fn idle_timeout(&self) -> Option<Duration> {
        match self.idle_timeout_seconds {
            0 => None,
            seconds => Some(Duration::from_secs(seconds))
        }
    }

    /// How long a broadcast waits between retries.
    pub fn broadcast_retry_delay(&self) -> Duration {
        Duration::from_millis(self.broadcast_retry_delay_ms)
    }
}

impl Default for Database {
    fn default() -> Self {
        Database {
            path: "chat.db".to_string(),
            max_connections: 5,
            min_connections: 0,
            acquire_timeout_seconds: 30,
            idle_timeout_seconds: 600,
            busy_timeout_seconds: 5,
            broadcast_retries: 2,
            broadcast_retry_delay_ms: 100,
        }
    }
}
