//! Failures and blocks counted per IP, and failed logins counted per account.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv6Addr};
use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::config;
use crate::error::{AppError, Result};
use crate::utils::mutex_lock;

/// Failures counted for one IP, and whether it is blocked
#[derive(Default)]
struct AddressRecord {
    register_failures: u32,
    login_failures: u32,
    blocked: bool
}

/// Failures and blocks per IP, with each IPv6 /64 counted as one IP
#[derive(Clone, Default)]
pub struct AddressLimits {
    by_ip: Arc<Mutex<HashMap<IpAddr, AddressRecord>>>
}

impl AddressLimits {

    /// Marks every IP read from the database as blocked
    pub fn load(&self, addresses: Vec<IpAddr>) {
        let mut by_ip = mutex_lock(&self.by_ip);

        // Mark each saved address as blocked
        for address in addresses {
            by_ip.entry(accountable_address(address)).or_default().blocked = true;
        }
    }

    /// Fails if the IP is blocked
    pub fn check(&self, ip: IpAddr) -> Result<()> {
        if mutex_lock(&self.by_ip).get(&accountable_address(ip)).is_some_and(|record| record.blocked) {
            return Err(AppError::AddressBlocked);
        }

        Ok(())
    }

    /// Counts a failed registration, returning whether the IP just reached the limit
    pub fn register_limit_reached(&self, ip: IpAddr) -> bool {
        let max = config::get().rate_limit.address_register_failures_max;
        let mut by_ip = mutex_lock(&self.by_ip);
        let record = by_ip.entry(accountable_address(ip)).or_default();
        record.register_failures += 1;

        if record.register_failures != max {
            return false;
        }

        tracing::warn!("| [{ip}] hit registration attempts limit");
        true
    }

    /// Counts a failed login, returning whether the IP just reached the limit
    pub fn login_limit_reached(&self, ip: IpAddr) -> bool {
        let max = config::get().rate_limit.address_login_failures_max;
        let mut by_ip = mutex_lock(&self.by_ip);
        let record = by_ip.entry(accountable_address(ip)).or_default();
        record.login_failures += 1;

        if record.login_failures != max {
            return false;
        }

        tracing::warn!("| [{ip}] hit login attempts limit");
        true
    }

    /// Blocks the IP
    pub fn block(&self, ip: IpAddr) {
        mutex_lock(&self.by_ip).entry(accountable_address(ip)).or_default().blocked = true;
    }

    /// Removes the IP's block and failure counts
    pub fn unblock(&self, ip: IpAddr) {
        mutex_lock(&self.by_ip).remove(&accountable_address(ip));
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
        let max = config::get().rate_limit.account_login_failures_max;
        let mut count_by_user = mutex_lock(&self.count_by_user);

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
        mutex_lock(&self.count_by_user).remove(&user_id);
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
