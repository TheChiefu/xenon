//! Signing in and out from the window

use rfd::{
    AsyncMessageDialog,
    MessageButtons,
    MessageDialog,
    MessageDialogResult,
    MessageLevel,
};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use client::accounts;
use client::auth;
use client::events;
use client::state::session;
use crate::gui::subscriptions::Subscribe;
use crate::gui::window::{App, Entry, Session};

const SIGN_OUT_TITLE: &str = "Sign Out";
const SIGN_OUT_QUESTION: &str = "Are you sure you want to sign out?";
const REMOVE_TITLE: &str = "Remove Account";
const REMOVE_QUESTION: &str = "Are you sure you want to remove this account?";

/// Restore the session
pub fn connect(app: &App) {
    auth::restore();

    events::SIGNED_IN.subscribe_window(app, show_signed_in);
    events::SIGNED_OUT.subscribe_window(app, show_signed_out);

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
fn sign_in(
    username: SharedString,
    password: SharedString,
    server: SharedString
) {
    tokio::spawn(auth::sign_in(
        username.to_string(),
        password.to_string(),
        server.to_string()
    ));
}

/// Start creating an account on a background thread
fn register(
    username: SharedString,
    password: SharedString,
    display_name: SharedString,
    code: SharedString,
    server: SharedString
) {
    tokio::spawn(auth::register(
        username.to_string(),
        password.to_string(),
        display_name.to_string(),
        code.to_string(),
        server.to_string(),
    ));
}

fn switch_account() {
    auth::switch_account();
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
    }
}

/// Get the saved accounts as rows for the login screen
fn accounts() -> ModelRc<Entry> {
    let mut entries = Vec::new();
    for (key, account) in accounts::list() {
        entries.push(Entry {
            key: key.into(),
            username: account.username.into(),
            server: account.server.into(),
        });
    }

    ModelRc::new(VecModel::from(entries))
}

fn select(key: SharedString) {
    auth::select(&key);
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

fn show_signed_in(app: &App) {
    app.global::<Session>().set_signed_in(true);
}

fn show_signed_out(app: &App) {
    app.global::<Session>().set_signed_in(false);
}
