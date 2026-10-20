//! Registration attempts counted per IP, and failed logins counted per account.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv6Addr};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use uuid::Uuid;

use crate::config;
use crate::error::{AppError, Result};

/// Registration attempts counted for one IP
struct Attempts {
    first_attempt: Instant,
    count: u32
}

/// Registration attempts per IP, with each IPv6 /64 counted as one IP
#[derive(Clone, Default)]
pub struct RegisterLimit {
    attempts_by_ip: Arc<Mutex<HashMap<IpAddr, Attempts>>>
}

impl RegisterLimit {

    /// Counts a registration attempt, failing once the IP has used all of its attempts
    pub fn check(&self, ip: IpAddr) -> Result<()> {
        let config = &config::get().rate_limit;
        let reset_after = Duration::from_secs(config.register_window_minutes * 60);
        let now = Instant::now();

        // Lock the map
        let mut attempts_by_ip = match self.attempts_by_ip.lock() {
            Ok(attempts_by_ip) => attempts_by_ip,
            Err(poisoned) => poisoned.into_inner(),
        };

        // Remove every IP whose first attempt was longer ago than the reset time
        attempts_by_ip.retain(|_, attempts| now.duration_since(attempts.first_attempt) < reset_after);

        // Count starts at zero on an IP's first attempt
        let attempts = attempts_by_ip.entry(accountable_address(ip)).or_insert(Attempts {
            first_attempt: now,
            count: 0
        });

        if attempts.count >= config.register_attempts_max {
            return Err(AppError::TooManyRequests);
        }

        attempts.count += 1;
        Ok(())
    }
}

/// Failed logins per account since its last successful login
#[derive(Clone, Default)]
pub struct LoginFailures {
    count_by_user: Arc<Mutex<HashMap<Uuid, u32>>>
}

impl LoginFailures {

    /// Counts a failed login, returning whether the account has reached its limit
    pub fn record_failure(&self, user_id: Uuid) -> bool {
        let max = config::get().rate_limit.login_failures_max;
        let mut count_by_user = self.lock_map();

        let count = count_by_user.entry(user_id).or_insert(0);
        *count += 1;

        // Limit reached, remove the count so an unlocked account starts again from zero
        if *count >= max {
            count_by_user.remove(&user_id);
            return true;
        }

        false
    }

    /// Removes an account's failed login count
    pub fn clear(&self, user_id: Uuid) {
        self.lock_map().remove(&user_id);
    }

    /// Locks the failed login counts
    fn lock_map(&self) -> MutexGuard<'_, HashMap<Uuid, u32>> {
        match self.count_by_user.lock() {
            Ok(count_by_user) => count_by_user,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

/// Address held accountable for an attempt, an IPv4 address as is or an IPv6 network
fn accountable_address(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V4(_) => ip,
        IpAddr::V6(address) => {

            // IPv6 address as a 128 bit number, first 64 bits are the network, last 64 bits are the device
            let bits = u128::from(address);

            // Ones over the 64 network bits, zeros over the 64 device bits
            let network_mask = u128::MAX << 64;

            // Mask out the device bits, holding the incoming network accountable for the attempt
            IpAddr::V6(Ipv6Addr::from(bits & network_mask))
        }
    }
}
