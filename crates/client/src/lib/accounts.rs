//! The accounts saved on this machine

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::events;

const ACCOUNTS_FILE: &str = "accounts.json";

// ACCOUNT //
#[derive(Clone, Serialize, Deserialize)]
pub struct Account {
    pub server: String,
    pub username: String,
    pub session: String,
    pub active: bool,
}

impl Account {

    /// Get the key the account is saved under
    pub fn key(&self) -> String {
        format!("{}@{}", self.username, self.server)
    }
}

// PUBLIC METHODS //

/// Get every saved account with its key
pub fn list() -> Vec<(String, Account)> {
    let mut listed = Vec::new();
    for (key, account) in read() {
        listed.push((key, account));
    }
    listed
}

/// Get the active account
pub fn active() -> Option<Account> {
    for account in read().into_values() {
        if account.active {
            return Some(account);
        }
    }
    None
}

/// Add or replace an account
pub fn add(account: Account) {
    let mut accounts = read();
    accounts.insert(account.key(), account);
    write(&accounts);
}

/// Remove an account
pub fn remove(key: &str) -> Option<Account> {
    let mut accounts = read();
    let removed = accounts.remove(key);
    write(&accounts);
    removed
}

/// Mark an account as active
pub fn activate(key: &str) -> Option<Account> {
    let mut accounts = read();
    let account = accounts.get_mut(key)?;
    account.active = true;
    let activated = account.clone();
    write(&accounts);
    Some(activated)
}

/// Mark an account as inactive
pub fn deactivate(key: &str) {
    let mut accounts = read();
    if let Some(account) = accounts.get_mut(key) {
        account.active = false;
    }
    write(&accounts);
}

// HELPERS //

/// Read the accounts file
fn read() -> BTreeMap<String, Account> {

    // Read the accounts file
    let text = match fs::read_to_string(path()) {
        Ok(text) => text,
        Err(_) => return BTreeMap::new(),
    };

    // Parse the accounts
    match serde_json::from_str(&text) {
        Ok(accounts) => accounts,
        Err(_) => BTreeMap::new(),
    }
}

/// Write the accounts file
fn write(accounts: &BTreeMap<String, Account>) {
    let text = serde_json::to_string_pretty(accounts).expect("saved accounts should serialize");

    if let Err(e) = fs::write(path(), text) {
        events::ERROR.publish(e.to_string());
    }
}

/// Get the path of the accounts file, next to the executable
fn path() -> PathBuf {
    let executable = env::current_exe().expect("the app should know where it runs from");
    executable.with_file_name(ACCOUNTS_FILE)
}
