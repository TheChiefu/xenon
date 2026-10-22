use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use tokio::sync::mpsc;
use uuid::Uuid;

use crate::config;
use crate::db;
use crate::error::Result;
use crate::sockets::events::{ActiveCall, CallServerEvent};
use crate::state::AppState;

// Data Structs //

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

impl ActiveCalls {

    fn read_lock(&self) -> RwLockReadGuard<'_, SignalRoutes> {
        match self.routes.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::warn!("calls lock was poisoned, recovering");
                poisoned.into_inner()
            }
        }
    }

    fn write_lock(&self) -> RwLockWriteGuard<'_, SignalRoutes> {
        match self.routes.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::warn!("calls lock was poisoned, recovering");
                poisoned.into_inner()
            }
        }
    }
}

// Call Methods //

/// Adds a user to a room's call
pub fn join(
    state: &AppState,
    room_id: Uuid,
    user_id: Uuid,
) -> std::result::Result<mpsc::Receiver<String>, JoinRefused> {
    let mut guard = state.calls.write_lock();

    // Borrow the routes out of the lock guard, so both maps can change at once
    let routes = &mut *guard;

    if routes.by_user.contains_key(&user_id) {
        return Err(JoinRefused::AlreadyInCall);
    }

    // The first participant starts the call
    let participants = routes.by_room.entry(room_id).or_default();
    if participants.len() >= config::get().calls.participants_max {
        return Err(JoinRefused::Full);
    }

    let (sender, receiver) = mpsc::channel(config::get().calls.signal_queue_size);
    participants.insert(user_id, sender);
    routes.by_user.insert(user_id, room_id);

    Ok(receiver)
}

/// Removes a user from their call, returning its room
pub fn leave(
    state: &AppState,
    user_id: Uuid,
) -> Option<Uuid> {
    let mut guard = state.calls.write_lock();

    // Borrow the routes out of the lock guard, so both maps can change at once
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
    state: &AppState,
    room_id: Uuid,
) {
    let mut guard = state.calls.write_lock();

    // Borrow the routes out of the lock guard, so both maps can change at once
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
    state: &AppState,
    room_id: Uuid,
) -> Vec<Uuid> {
    let routes = state.calls.read_lock();

    let mut found = Vec::new();
    if let Some(participants) = routes.by_room.get(&room_id) {
        for user_id in participants.keys() {
            found.push(*user_id);
        }
    }

    found
}

/// Builds the list of active calls in every room a user is a member of
pub async fn snapshot(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<ActiveCall>> {
    let mut conn = state.pool.acquire().await?;
    let room_ids = db::member_room_ids(&mut conn, user_id).await?;

    let routes = state.calls.read_lock();

    let mut calls = Vec::new();
    for room_id in room_ids {

        // Skip rooms with no call
        let Some(participants) = routes.by_room.get(&room_id) else {
            continue;
        };

        let mut user_ids = Vec::new();
        for participant_id in participants.keys() {
            user_ids.push(*participant_id);
        }

        calls.push(ActiveCall { room_id, participants: user_ids });
    }

    Ok(calls)
}

/// Reads the room id of the call a participant is in
pub fn participant_call(
    state: &AppState,
    user_id: Uuid,
) -> Option<Uuid> {
    let routes = state.calls.read_lock();

    match routes.by_user.get(&user_id) {
        Some(room_id) => Some(*room_id),
        None => None
    }
}

/// Passes a signal to its target when both are in the same call, and drops it otherwise
pub fn forward(
    state: &AppState,
    from: Uuid,
    to: Uuid,
    payload: String,
) {
    let event = CallServerEvent::Signal { from, payload };
    let text = match serde_json::to_string(&event) {
        Ok(text) => text,
        Err(e) => {
            tracing::error!("failed to serialize call signal: {e}");
            return;
        }
    };

    let routes = state.calls.read_lock();

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
