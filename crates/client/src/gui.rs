//! The window's side of the client

pub mod activity;
pub mod errors;
pub mod subscriptions;
pub mod identity;
pub mod session;

// Private, so only the window's side can reach the generated Slint types
mod window {
    slint::include_modules!();
}

use slint::ComponentHandle;

use client::state::profile;
use crate::gui::window::App;

/// Open the window and run it until it closes
pub fn run() -> Result<(), slint::PlatformError> {
    let app = App::new()?;

    profile::subscribe();
    activity::subscribe(&app);
    errors::subscribe(&app);
    identity::subscribe(&app);
    session::connect(&app);

    app.run()
}
