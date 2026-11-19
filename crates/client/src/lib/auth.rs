use types::auth::Registration;

use crate::accounts::Account;
use crate::api::{self, ErrorMessage};
use crate::events::{self, Event};
use crate::state::profile;
use crate::state::session;

// PUBLIC METHODS //

/// Log in on the server and make the account the active one
pub async fn sign_in(
    username: String,
    password: String,
    server: String
) -> Result<(), ErrorMessage> {
    let server = server.trim_end_matches('/').to_string();
    let token = api::login(&server, &username, &password).await?;

    session::add(&Account { server, username, session: token });
    tokio::spawn(begin());
    Ok(())
}

/// Create an account on the server and make it the active one
pub async fn register(
    username: String,
    password: String,
    display_name: String,
    code: String,
    server: String
) -> Result<(), ErrorMessage> {
    let server = server.trim_end_matches('/').to_string();
    let registration = Registration {
        registration_code: Some(code),
        username: username.clone(),
        password,
        display_name,
    };
    let token = api::register(&server, &registration).await?;

    session::add(&Account { server, username, session: token });
    tokio::spawn(begin());
    Ok(())
}

/// Sign in with a saved account's existing session
pub fn select(key: &str) -> bool {
    if !session::activate(key) {
        return false;
    }

    tokio::spawn(begin());
    true
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
}

/// Remove the active account and end its session on the server
pub fn sign_out() {
    if let Some(account) = session::remove_active() {
        tokio::spawn(end_session(account));
    }
}

/// Remove a saved account and end its session on the server
pub fn remove(key: &str) {
    if let Some(account) = session::remove(key) {
        tokio::spawn(end_session(account));
    }
}

/// Load the signed-in session's data
pub async fn begin() {
    events::dispatch(Event::Working);

    match profile::load().await {
        Ok(()) => events::dispatch(Event::Done),
        Err(reason) => events::dispatch(Event::Error(reason)),
    }
}

// HELPERS //

async fn end_session(account: Account) {
    if let Err(reason) = api::sign_out(&account.server, &account.session).await {
        events::dispatch(Event::Error(reason));
    }
}
