//! The window's side of the client

pub mod activity;
pub mod errors;
pub mod identity;
pub mod session;

// Private, so only the window's side can reach the generated Slint types
mod window {
    slint::include_modules!();
}

use std::sync::OnceLock;

use slint::{ComponentHandle, Weak};

use client::events::Listeners;
use crate::gui::window::App;

static WINDOW: OnceLock<Weak<App>> = OnceLock::new();

/// Open the window and run it until it closes
pub fn run() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    if WINDOW.set(app.as_weak()).is_err() {
        panic!("the window should only open once");
    }

    let mut listeners = Listeners::new();
    activity::subscribe(&app, &mut listeners);
    errors::subscribe(&app, &mut listeners);
    identity::subscribe(&app, &mut listeners);
    listeners.start();

    session::connect(&app);
    app.run()
}
