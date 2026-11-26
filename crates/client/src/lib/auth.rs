use types::auth::Registration;

use crate::accounts::{self, Account};
use crate::api;
use crate::events;
use crate::state::profile;
use crate::state::server;
use crate::state::session;

// PUBLIC METHODS //

/// Log in on the server and make the account the active one
pub async fn sign_in(
    username: String,
    password: String,
    server: String
) {
    events::WORKING.publish();

    let server = server.trim_end_matches('/').to_string();
    match api::login(&server, &username, &password).await {
        Ok(token) => {
            let account = Account { server, username, session: token, active: true };
            accounts::add(account.clone());
            session::start(account);
            tokio::spawn(begin());
            events::DONE.publish();
            events::SIGNED_IN.publish();
        }
        Err(reason) => events::ERROR.publish(reason),
    }
}

/// Create an account on the server and make it the active one
pub async fn register(
    username: String,
    password: String,
    display_name: String,
    code: String,
    server: String
) {
    events::WORKING.publish();

    let server = server.trim_end_matches('/').to_string();
    let registration = Registration {
        registration_code: Some(code),
        username: username.clone(),
        password,
        display_name,
    };
    match api::register(&server, &registration).await {
        Ok(token) => {
            let account = Account { server, username, session: token, active: true };
            accounts::add(account.clone());
            session::start(account);
            tokio::spawn(begin());
            events::DONE.publish();
            events::SIGNED_IN.publish();
        }
        Err(reason) => events::ERROR.publish(reason),
    }
}

/// Sign in with a saved account's existing session
pub fn select(key: &str) {
    let Some(account) = accounts::activate(key) else {
        return;
    };

    session::start(account);
    tokio::spawn(begin());
    events::SIGNED_IN.publish();
}

/// Restore the last active account
pub fn restore() {
    if let Some(account) = accounts::active() {
        session::start(account);
        tokio::spawn(begin());
    }
}

/// Stop using the active account, keeping it saved
pub fn switch_account() {
    if let Some(account) = session::active_account() {
        accounts::deactivate(&account.key());
    }
    session::end();
    events::SIGNED_OUT.publish();
}

/// Remove the active account and end its session on the server
pub fn sign_out() {
    if let Some(account) = session::active_account() {
        accounts::remove(&account.key());
        tokio::spawn(end_session(account));
    }
    session::end();
    events::SIGNED_OUT.publish();
}

/// Remove a saved account and end its session on the server
pub fn remove(key: &str) {
    if let Some(account) = accounts::remove(key) {
        tokio::spawn(end_session(account));
    }
}

/// Load the signed-in session's data
pub async fn begin() {
    events::WORKING.publish();

    tokio::spawn(load_server());

    match profile::load().await {
        Ok(()) => events::DONE.publish(),
        Err(reason) => events::ERROR.publish(reason),
    }
}

// HELPERS //

/// Load the session's server information
async fn load_server() {
    let Some(account) = session::active_account() else {
        return;
    };

    match api::server_info(&account.server).await {
        Ok(info) => server::set(Some(info)),
        Err(reason) => events::ERROR.publish(reason),
    }
}

async fn end_session(account: Account) {
    if let Err(reason) = api::sign_out(&account.server, &account.session).await {
        events::ERROR.publish(reason);
    }
}
