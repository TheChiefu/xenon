//! The accounts saved on this machine

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::events;

const ACCOUNTS_FILE: &str = "accounts.json";

// TYPES //
#[derive(Clone, Serialize, Deserialize)]
pub struct Account {
    pub server: String,
    pub username: String,
    pub session: String,
}

#[derive(Serialize, Deserialize)]
pub struct Saved {
    accounts: BTreeMap<String, Account>, // Keyed by "username@server"
    active: Option<String>, // Key of the active account (when available)
}

// METHODS //
impl Saved {

    /// Get a store with no accounts
    pub const fn empty() -> Self {
        Saved {
            accounts: BTreeMap::new(),
            active: None,
        }
    }

    pub fn load() -> Self {

        // Read the accounts file
        let text = match fs::read_to_string(path()) {
            Ok(text) => text,
            Err(_) => return Saved::empty(),
        };

        // Parse the accounts
        match serde_json::from_str(&text) {
            Ok(saved) => saved,
            Err(_) => Saved::empty(),
        }
    }

    /// Write the saved accounts
    pub fn save(&self) {
        let text = serde_json::to_string_pretty(self).expect("saved accounts should serialize");

        if let Err(e) = fs::write(path(), text) {
            events::ERROR.publish(e.to_string());
        }
    }

    /// Add or replace the account and make it the active one
    pub fn update(&mut self, account: &Account) {
        let key = format!("{}@{}", account.username, account.server);
        self.accounts.insert(key.clone(), account.clone());
        self.active = Some(key);
    }

    /// Get the active account
    pub fn active_account(&self) -> Option<Account> {
        match &self.active {
            Some(key) => self.accounts.get(key).cloned(),
            None => None,
        }
    }

    /// Get every saved account with its key
    pub fn list(&self) -> Vec<(String, Account)> {
        let mut listed = Vec::new();
        for (key, account) in &self.accounts {
            listed.push((key.clone(), account.clone()));
        }
        listed
    }

    /// Make a saved account the active one, if it exists
    pub fn activate(&mut self, key: &str) -> bool {
        if !self.accounts.contains_key(key) {
            return false;
        }

        self.active = Some(key.to_string());
        true
    }

    /// Stop using the active account
    pub fn deactivate(&mut self) {
        self.active = None;
    }

    /// Remove a saved account
    pub fn remove(&mut self, key: &str) -> Option<Account> {
        if let Some(active) = &self.active {
            if active == key {
                self.active = None;
            }
        }

        self.accounts.remove(key)
    }

    /// Remove the active account
    pub fn remove_active(&mut self) -> Option<Account> {
        match self.active.take() {
            Some(key) => self.accounts.remove(&key),
            None => None,
        }
    }
}

// HELPERS //

/// Get the path of the accounts file, next to the executable
fn path() -> PathBuf {
    let executable = env::current_exe().expect("the app should know where it runs from");
    executable.with_file_name(ACCOUNTS_FILE)
}
