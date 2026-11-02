//! Who is connected over RTC in each room

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use tokio::sync::Mutex;
use uuid::Uuid;

use crate::config;
use crate::rtc::events::{RtcCloseCode, RtcRoom};
use crate::rtc::peer::Peer;
use crate::utils;

// DATA STRUCTS //

#[derive(Default)]
struct Rooms {
    /// Peers connected in each room, keyed by user id
    by_id: HashMap<Uuid, HashMap<Uuid, Arc<Peer>>>,

    /// The room a user is connected to
    by_user: HashMap<Uuid, Uuid>
}

#[derive(Clone, Default)]
pub struct Connections {
    rooms: Arc<RwLock<Rooms>>,

    /// Lets one join or leave at a time add or remove tracks
    pub track_lock: Arc<Mutex<()>>
}

// PUBLIC METHODS //

/// Add a peer to its room, returning the peers already there, or the code to refuse it with
pub fn insert(connections: &Connections, peer: Arc<Peer>) -> Result<Vec<Arc<Peer>>, RtcCloseCode> {
    let mut guard = utils::write_lock(&connections.rooms);
    let rooms = &mut *guard;

    let room = rooms.by_id.entry(peer.room_id).or_default();
    if room.len() >= config::get().sockets.participants_max {
        return Err(RtcCloseCode::Full);
    }

    let others = room.values().cloned().collect();

    room.insert(peer.user_id, peer.clone());
    rooms.by_user.insert(peer.user_id, peer.room_id);

    Ok(others)
}

/// Remove a peer, returning the peers left in its room.
/// None when the peer is no longer the one connected for its user.
pub fn remove(connections: &Connections, peer: &Arc<Peer>) -> Option<Vec<Arc<Peer>>> {
    let mut guard = utils::write_lock(&connections.rooms);
    let rooms = &mut *guard;

    let room = rooms.by_id.get_mut(&peer.room_id)?;
    let current = room.get(&peer.user_id)?;
    if !Arc::ptr_eq(current, peer) {
        return None;
    }

    room.remove(&peer.user_id);
    let remaining = room.values().cloned().collect();

    if room.is_empty() {
        rooms.by_id.remove(&peer.room_id);
    }

    rooms.by_user.remove(&peer.user_id);

    Some(remaining)
}

pub fn peer(connections: &Connections, user_id: Uuid) -> Option<Arc<Peer>> {
    let rooms = utils::read_lock(&connections.rooms);
    let room_id = rooms.by_user.get(&user_id)?;
    rooms.by_id.get(room_id)?.get(&user_id).cloned()
}

pub fn in_room(connections: &Connections, room_id: Uuid) -> Vec<Arc<Peer>> {
    let rooms = utils::read_lock(&connections.rooms);
    match rooms.by_id.get(&room_id) {
        Some(room) => room.values().cloned().collect(),
        None => Vec::new()
    }
}

/// Build who is connected in the given rooms, skipping rooms nobody is connected in
pub fn snapshot(connections: &Connections, room_ids: &[Uuid]) -> Vec<RtcRoom> {
    let rooms = utils::read_lock(&connections.rooms);

    let mut snapshot = Vec::new();
    for room_id in room_ids {
        let Some(room) = rooms.by_id.get(room_id) else {
            continue;
        };

        snapshot.push(RtcRoom {
            room_id: *room_id,
            participants: room.keys().copied().collect()
        });
    }

    snapshot
}
