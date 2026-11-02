//! Who is connected over RTC in each room, and the UDP ports their connections hold

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use tokio::sync::Mutex;
use uuid::Uuid;

use crate::config;
use crate::rtc::events::RtcRoom;
use crate::rtc::peer::Peer;
use crate::utils;

// DATA STRUCTS //

#[derive(Default)]
struct Connections {
    /// Peers connected in each room, keyed by user id
    by_room: HashMap<Uuid, HashMap<Uuid, Arc<Peer>>>,

    /// The room a user is connected to
    by_user: HashMap<Uuid, Uuid>,

    /// The UDP port a user's connection holds
    port_of: HashMap<Uuid, u16>,

    /// UDP ports held, including ports reserved for a connection still opening
    ports: HashSet<u16>
}

#[derive(Clone, Default)]
pub struct Registry {
    connections: Arc<RwLock<Connections>>,

    /// Lets one join or leave at a time add or remove tracks
    pub track_lock: Arc<Mutex<()>>
}

// PUBLIC METHODS //

/// Reserve the next unused port in the configured range
pub fn take_port(registry: &Registry) -> Option<u16> {
    let mut connections = utils::write_lock(&registry.connections);

    let port = config::get().sockets.udp_ports.clone()
        .find(|port| !connections.ports.contains(port))?;

    connections.ports.insert(port);
    Some(port)
}

pub fn release_port(registry: &Registry, port: u16) {
    utils::write_lock(&registry.connections).ports.remove(&port);
}

/// Add a peer holding a port to its room, returning the peers already there,
/// or None when the room is full
pub fn insert(registry: &Registry, peer: Arc<Peer>, port: u16) -> Option<Vec<Arc<Peer>>> {
    let mut guard = utils::write_lock(&registry.connections);
    let connections = &mut *guard;

    let room = connections.by_room.entry(peer.room_id).or_default();
    if room.len() >= config::get().sockets.participants_max {
        return None;
    }

    let others = room.values().cloned().collect();

    room.insert(peer.user_id, peer.clone());
    connections.by_user.insert(peer.user_id, peer.room_id);
    connections.port_of.insert(peer.user_id, port);

    Some(others)
}

/// Remove a peer and release its port, returning the peers left in its room.
/// None when the peer is no longer the one connected for its user.
pub fn remove(registry: &Registry, peer: &Arc<Peer>) -> Option<Vec<Arc<Peer>>> {
    let mut guard = utils::write_lock(&registry.connections);
    let connections = &mut *guard;

    let room = connections.by_room.get_mut(&peer.room_id)?;
    let current = room.get(&peer.user_id)?;
    if !Arc::ptr_eq(current, peer) {
        return None;
    }

    room.remove(&peer.user_id);
    let remaining = room.values().cloned().collect();

    if room.is_empty() {
        connections.by_room.remove(&peer.room_id);
    }

    connections.by_user.remove(&peer.user_id);
    if let Some(port) = connections.port_of.remove(&peer.user_id) {
        connections.ports.remove(&port);
    }

    Some(remaining)
}

pub fn peer(registry: &Registry, user_id: Uuid) -> Option<Arc<Peer>> {
    let connections = utils::read_lock(&registry.connections);
    let room_id = connections.by_user.get(&user_id)?;
    connections.by_room.get(room_id)?.get(&user_id).cloned()
}

pub fn peers(registry: &Registry, room_id: Uuid) -> Vec<Arc<Peer>> {
    let connections = utils::read_lock(&registry.connections);
    match connections.by_room.get(&room_id) {
        Some(room) => room.values().cloned().collect(),
        None => Vec::new()
    }
}

/// Build who is connected in the given rooms, skipping rooms nobody is connected in
pub fn snapshot(registry: &Registry, room_ids: &[Uuid]) -> Vec<RtcRoom> {
    let connections = utils::read_lock(&registry.connections);

    let mut rooms = Vec::new();
    for room_id in room_ids {
        let Some(room) = connections.by_room.get(room_id) else {
            continue;
        };

        rooms.push(RtcRoom {
            room_id: *room_id,
            participants: room.keys().copied().collect()
        });
    }

    rooms
}
