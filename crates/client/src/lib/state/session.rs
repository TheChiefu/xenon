//! The signed-in session

use std::sync::Mutex;

use crate::accounts::Account;

const LOCK_FAILED: &str = "session should not be left locked by a crash";

static ACTIVE: Mutex<Option<Account>> = Mutex::new(None);

/// Start a session with the account
pub fn start(account: Account) {
    let mut state = ACTIVE.lock().expect(LOCK_FAILED);
    *state = Some(account);
}

/// End the session
pub fn end() {
    let mut state = ACTIVE.lock().expect(LOCK_FAILED);
    state.take();
}

/// Get the signed-in account, if any
pub fn active_account() -> Option<Account> {
    let state = ACTIVE.lock().expect(LOCK_FAILED);
    state.clone()
}
