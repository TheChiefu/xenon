use types::auth::Registration;

use crate::accounts::Account;
use crate::api;
use crate::events;
use crate::state::profile;
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
            session::add(&Account { server, username, session: token });
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
            session::add(&Account { server, username, session: token });
            tokio::spawn(begin());
            events::DONE.publish();
            events::SIGNED_IN.publish();
        }
        Err(reason) => events::ERROR.publish(reason),
    }
}

/// Sign in with a saved account's existing session
pub fn select(key: &str) {
    if !session::activate(key) {
        return;
    }

    tokio::spawn(begin());
    events::SIGNED_IN.publish();
}

/// Restore the last active account
pub fn restore() {
    session::load();

    if session::active_account().is_some() {
        tokio::spawn(begin());
    }
}

/// Stop using the active account, keeping it saved
pub fn switch_account() {
    session::deactivate();
    events::SIGNED_OUT.publish();
}

/// Remove the active account and end its session on the server
pub fn sign_out() {
    if let Some(account) = session::remove_active() {
        tokio::spawn(end_session(account));
    }
    events::SIGNED_OUT.publish();
}

/// Remove a saved account and end its session on the server
pub fn remove(key: &str) {
    if let Some(account) = session::remove(key) {
        tokio::spawn(end_session(account));
    }
}

/// Load the signed-in session's data
pub async fn begin() {
    events::WORKING.publish();

    match profile::load().await {
        Ok(()) => events::DONE.publish(),
        Err(reason) => events::ERROR.publish(reason),
    }
}

// HELPERS //

async fn end_session(account: Account) {
    if let Err(reason) = api::sign_out(&account.server, &account.session).await {
        events::ERROR.publish(reason);
    }
}
