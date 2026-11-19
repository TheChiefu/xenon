//! Signing in and out from the window

use rfd::{
    AsyncMessageDialog,
    MessageButtons,
    MessageDialog,
    MessageDialogResult,
    MessageLevel,
};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use client::auth;
use client::events::{self, Event};
use client::state::session;
use crate::gui::WINDOW;
use crate::gui::window::{App, Entry, Session};

const SIGN_OUT_TITLE: &str = "Sign Out";
const SIGN_OUT_QUESTION: &str = "Are you sure you want to sign out?";
const REMOVE_TITLE: &str = "Remove Account";
const REMOVE_QUESTION: &str = "Are you sure you want to remove this account?";

/// Restore the session
pub fn connect(app: &App) {
    auth::restore();

    let actions = app.global::<Session>();
    actions.set_signed_in(session::active_account().is_some());
    actions.on_sign_in(sign_in);
    actions.on_register(register);
    actions.on_switch_account(switch_account);
    actions.on_sign_out(sign_out);
    actions.on_accounts(accounts);
    actions.on_select(select);
    actions.on_remove(remove);
}

// ACTIONS //

/// Start signing in on a background thread
fn sign_in(username: SharedString, password: SharedString, server: SharedString) {
    events::dispatch(Event::Working);
    tokio::spawn(finish_sign_in(username.to_string(), password.to_string(), server.to_string()));
}

async fn finish_sign_in(username: String, password: String, server: String) {
    match auth::sign_in(username, password, server).await {
        Ok(()) => {
            events::dispatch(Event::Done);
            set_signed_in(true);
        }
        Err(reason) => events::dispatch(Event::Error(reason)),
    }
}

/// Start creating an account on a background thread
fn register(
    username: SharedString,
    password: SharedString,
    display_name: SharedString,
    code: SharedString,
    server: SharedString
) {
    events::dispatch(Event::Working);
    tokio::spawn(finish_register(
        username.to_string(),
        password.to_string(),
        display_name.to_string(),
        code.to_string(),
        server.to_string(),
    ));
}

async fn finish_register(
    username: String,
    password: String,
    display_name: String,
    code: String,
    server: String
) {
    match auth::register(username, password, display_name, code, server).await {
        Ok(()) => {
            events::dispatch(Event::Done);
            set_signed_in(true);
        }
        Err(reason) => events::dispatch(Event::Error(reason)),
    }
}

fn switch_account() {
    auth::switch_account();
    set_signed_in(false);
}

/// Ask to confirm before signing out
fn sign_out() {
    tokio::spawn(confirm_sign_out());
}

async fn confirm_sign_out() {
    let answer = AsyncMessageDialog::new()
        .set_title(SIGN_OUT_TITLE)
        .set_description(SIGN_OUT_QUESTION)
        .set_buttons(MessageButtons::YesNo)
        .set_level(MessageLevel::Warning)
        .show()
        .await;

    if let MessageDialogResult::Yes = answer {
        auth::sign_out();
        set_signed_in(false);
    }
}

/// Get the saved accounts as rows for the login screen
fn accounts() -> ModelRc<Entry> {
    let mut entries = Vec::new();
    for (key, account) in session::accounts() {
        entries.push(Entry {
            key: key.into(),
            username: account.username.into(),
            server: account.server.into(),
        });
    }

    ModelRc::new(VecModel::from(entries))
}

fn select(key: SharedString) {
    if auth::select(&key) {
        set_signed_in(true);
    }
}

/// Ask to confirm before removing a saved account
fn remove(key: SharedString) {

    // Blocking
    let answer = MessageDialog::new()
        .set_title(REMOVE_TITLE)
        .set_description(REMOVE_QUESTION)
        .set_buttons(MessageButtons::YesNo)
        .set_level(MessageLevel::Warning)
        .show();

    if let MessageDialogResult::Yes = answer {
        auth::remove(&key);
    }
}

// HELPERS //

/// Set which screen the window shows
fn set_signed_in(signed_in: bool) {
    if let Some(window) = WINDOW.get() {
        let _ = window.upgrade_in_event_loop(move |app| {
            app.global::<Session>().set_signed_in(signed_in);
        });
    }
}
