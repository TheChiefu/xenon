//! The signed-in session

use std::sync::Mutex;

use crate::accounts::{Account, Saved};

const LOCK_FAILED: &str = "saved accounts should not be left locked by a crash";

static STATE: Mutex<Saved> = Mutex::new(Saved::empty());

/// Load the saved accounts
pub fn load() {
    let mut state = STATE.lock().expect(LOCK_FAILED);
    *state = Saved::load();
}

/// Get the signed-in account, if any
pub fn active_account() -> Option<Account> {
    let state = STATE.lock().expect(LOCK_FAILED);
    state.active_account()
}

/// Get every saved account with its key
pub fn accounts() -> Vec<(String, Account)> {
    let state = STATE.lock().expect(LOCK_FAILED);
    state.list()
}

/// Add or replace an account and make it the active one
pub fn add(account: &Account) {
    let mut state = STATE.lock().expect(LOCK_FAILED);
    state.update(account);
    state.save();
}

/// Make a saved account the active one, if it exists
pub fn activate(key: &str) -> bool {
    let mut state = STATE.lock().expect(LOCK_FAILED);
    if !state.activate(key) {
        return false;
    }

    state.save();
    true
}

/// Stop using the active account, keeping it saved
pub fn deactivate() {
    let mut state = STATE.lock().expect(LOCK_FAILED);
    state.deactivate();
    state.save();
}

/// Remove the active account
pub fn remove_active() -> Option<Account> {
    let mut state = STATE.lock().expect(LOCK_FAILED);
    let removed = state.remove_active();
    state.save();
    removed
}

/// Remove a saved account
pub fn remove(key: &str) -> Option<Account> {
    let mut state = STATE.lock().expect(LOCK_FAILED);
    let removed = state.remove(key);
    state.save();
    removed
}
