//! Who is in each room's call, and the queues carrying their signaling messages

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use tokio::sync::mpsc;
use uuid::Uuid;

use crate::config;
use crate::sockets::call::events::CallServerEvent;
use crate::types::calls::RoomCall;
use crate::utils;

// DATA STRUCTS //

/// Carries WebRTC signaling messages (SDP offers and answers, ICE candidates)
/// from other participants to this participant's call socket
type SignalQueue = mpsc::Sender<String>;

/// A call's participants, keyed by user id
type Participants = HashMap<Uuid, SignalQueue>;

#[derive(Debug, PartialEq, Eq)]
pub enum JoinRefused {
    AlreadyInCall,
    Full
}

#[derive(Default)]
struct SignalRoutes {
    /// Each call's participants, keyed by room id
    by_room: HashMap<Uuid, Participants>,

    /// Each participant's call, as its room id, keyed by user id
    by_user: HashMap<Uuid, Uuid>
}

/// Every active call, and the room each participating user's call is in
#[derive(Clone, Default)]
pub struct ActiveCalls {
    routes: Arc<RwLock<SignalRoutes>>
}

// PUBLIC METHODS //

/// Adds a user to a room's call
pub fn join(
    calls: &ActiveCalls,
    room_id: Uuid,
    user_id: Uuid,
) -> Result<mpsc::Receiver<String>, JoinRefused> {
    let mut guard = utils::write_lock(&calls.routes);

    let routes = &mut *guard;

    if routes.by_user.contains_key(&user_id) {
        return Err(JoinRefused::AlreadyInCall);
    }

    // The first participant starts the call
    let participants = routes.by_room.entry(room_id).or_default();
    if participants.len() >= config::get().sockets.participants_max {
        return Err(JoinRefused::Full);
    }

    let (sender, receiver) = mpsc::channel(config::get().sockets.signal_queue_size);
    participants.insert(user_id, sender);
    routes.by_user.insert(user_id, room_id);

    Ok(receiver)
}

/// Removes a user from their call, returning its room
pub fn leave(
    calls: &ActiveCalls,
    user_id: Uuid,
) -> Option<Uuid> {
    let mut guard = utils::write_lock(&calls.routes);

    let routes = &mut *guard;

    let Some(room_id) = routes.by_user.remove(&user_id) else {
        return None;
    };

    if let Some(participants) = routes.by_room.get_mut(&room_id) {
        participants.remove(&user_id);

        // The last participant leaving ends the call
        if participants.is_empty() {
            routes.by_room.remove(&room_id);
        }
    }

    Some(room_id)
}

/// Removes everyone from a room's call
pub fn end(
    calls: &ActiveCalls,
    room_id: Uuid,
) {
    let mut guard = utils::write_lock(&calls.routes);

    let routes = &mut *guard;

    let Some(participants) = routes.by_room.remove(&room_id) else {
        return;
    };

    for user_id in participants.keys() {
        routes.by_user.remove(user_id);
    }
}

/// Reads who is in a room's call
pub fn participants(
    calls: &ActiveCalls,
    room_id: Uuid,
) -> Vec<Uuid> {
    let routes = utils::read_lock(&calls.routes);

    let mut user_ids = Vec::new();
    if let Some(participants) = routes.by_room.get(&room_id) {
        for user_id in participants.keys() {
            user_ids.push(*user_id);
        }
    }

    user_ids
}

/// Build list of active calls in all given rooms
pub fn snapshot(
    active_calls: &ActiveCalls,
    room_ids: &[Uuid],
) -> Vec<RoomCall> {
    let routes = utils::read_lock(&active_calls.routes);

    let mut room_calls = Vec::new();
    for room_id in room_ids {

        // Skip rooms with no call
        let Some(participants) = routes.by_room.get(room_id) else {
            continue;
        };

        let mut user_ids = HashSet::new();
        for participant_id in participants.keys() {
            user_ids.insert(*participant_id);
        }

        room_calls.push(
            RoomCall {
                room_id: *room_id,
                participants: user_ids
            }
        );
    }

    room_calls
}

/// Read the room id of the call a participant is in
pub fn participant_call(
    calls: &ActiveCalls,
    user_id: Uuid,
) -> Option<Uuid> {
    let routes = utils::read_lock(&calls.routes);

    match routes.by_user.get(&user_id) {
        Some(room_id) => Some(*room_id),
        None => None
    }
}

/// Pass a signal to a target when both are in the same call, and drop it otherwise
pub fn forward(
    calls: &ActiveCalls,
    from: Uuid,
    to: Uuid,
    payload: String,
) {
    let event = CallServerEvent::Signal { from, payload };
    let Some(text) = crate::sockets::encode(&event) else {
        return;
    };

    let routes = utils::read_lock(&calls.routes);

    // The sender must be in a call
    let Some(room_id) = routes.by_user.get(&from) else {
        return;
    };

    let Some(participants) = routes.by_room.get(room_id) else {
        return;
    };

    // The target must be in the sender's call
    let Some(queue) = participants.get(&to) else {
        return;
    };

    // A full queue drops the signal
    let _ = queue.try_send(text);
}
