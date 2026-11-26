//! The connected server's information

use std::sync::Mutex;

use types::routes::server::ServerInfo;

use crate::events;

const LOCK_FAILED: &str = "server info should not be left locked by a crash";

static INFO: Mutex<Option<ServerInfo>> = Mutex::new(None);

/// Get the server's information, if loaded
pub fn get() -> Option<ServerInfo> {
    INFO.lock().expect(LOCK_FAILED).clone()
}

/// Replace the server's information
pub fn set(info: Option<ServerInfo>) {
    *INFO.lock().expect(LOCK_FAILED) = info;
    events::SERVER_CHANGED.publish();
}

/// Determine how subscribed events are handled
pub fn subscribe() {
    events::SIGNED_OUT.subscribe(clear);
}

fn clear() {
    set(None);
}
