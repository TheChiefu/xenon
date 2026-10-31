//! The map of connected users, and sending events to them

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use tokio::sync::broadcast;
use uuid::Uuid;

use crate::config;
use crate::sockets::session::events::ServerEvent;
use crate::types::appearance::Device;
use crate::utils;

// DATA STRUCTS //

struct Connected {
    events: broadcast::Sender<String>,
    devices: Vec<SocketDevice>
}

pub struct SocketDevice {
    pub socket_id: u64,
    pub device: Device
}

/// Every connected user's channel, keyed by user id
#[derive(Clone, Default)]
pub struct Registry {
    connected: Arc<RwLock<HashMap<Uuid, Connected>>>
}

/// Next id handed to a socket
static NEXT_SOCKET_ID: AtomicU64 = AtomicU64::new(0);

// REGISTRY METHODS //

/// Take the next socket id
pub fn next_socket_id() -> u64 {
    NEXT_SOCKET_ID.fetch_add(1, Ordering::Relaxed)
}

/// Return a receiver on a user's channel, creating the channel on a first connection
/// and recording the socket's device
pub fn subscribe(
    registry: &Registry,
    user_id: Uuid,
    device: Option<SocketDevice>,
) -> broadcast::Receiver<String> {
    let mut reg = utils::write_lock(&registry.connected);

    let connected = match reg.entry(user_id) {

        // The user already has a channel
        Entry::Occupied(entry) => entry.into_mut(),

        // The user has no channel, create one
        Entry::Vacant(entry) => {
            let buffer = config::get().sockets.event_queue_size;
            let (sender, _) = broadcast::channel(buffer);
            entry.insert(Connected { events: sender, devices: Vec::new() })
        }
    };

    if let Some(device) = device {
        connected.devices.push(device);
    }

    connected.events.subscribe()
}

/// Drop a receiver and the user's entry with it once no other receiver remains
pub fn unsubscribe(
    registry: &Registry,
    user_id: Uuid,
    socket_id: u64,
    receiver: broadcast::Receiver<String>,
) {
    drop(receiver);

    let mut reg = utils::write_lock(&registry.connected);

    let connected = match reg.get_mut(&user_id) {
        Some(connected) => connected,
        None => return,
    };

    // Remove the device record this socket added, if it added one
    for i in 0..connected.devices.len() {
        if connected.devices[i].socket_id == socket_id {
            connected.devices.remove(i);
            break;
        }
    }

    // The user still has other sockets open
    if connected.events.receiver_count() > 0 {
        return;
    }

    // No sockets are left, remove the user
    reg.remove(&user_id);
}

/// Read whether a user holds any socket
pub fn is_connected(
    registry: &Registry,
    user_id: Uuid,
) -> bool {
    let reg = utils::read_lock(&registry.connected);
    reg.contains_key(&user_id)
}

/// Read the device a user most recently connected from
pub fn device(
    registry: &Registry,
    user_id: Uuid,
) -> Option<Device> {
    let reg = utils::read_lock(&registry.connected);

    let connected = reg.get(&user_id)?;
    match connected.devices.last() {
        Some(last) => Some(last.device),
        None => None
    }
}

/// Remove a user from the registry
pub fn remove(
    registry: &Registry,
    user_id: Uuid,
) {
    let mut reg = utils::write_lock(&registry.connected);
    reg.remove(&user_id);
}

// SENDING METHODS //

/// Send an event to a list of users
pub fn inform_users(
    registry: &Registry,
    users: &[Uuid],
    event: ServerEvent,
) {
    let Some(payload) = crate::sockets::encode(&event) else {
        return;
    };

    let reg = utils::read_lock(&registry.connected);

    for user_id in users {
        if let Some(connected) = reg.get(user_id) {
            let _ = connected.events.send(payload.clone());
        }
    }
}

/// Read which of the given users are not connected
pub fn offline_users(
    registry: &Registry,
    users: &[Uuid],
) -> Vec<Uuid> {
    let reg = utils::read_lock(&registry.connected);

    let mut found = Vec::new();
    for user_id in users {
        if !reg.contains_key(user_id) {
            found.push(*user_id);
        }
    }

    found
}
