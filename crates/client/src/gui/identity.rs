//! Shows who is signed in on the toolbar's identity card

use slint::ComponentHandle;

use client::events::{EventKind, Listeners};
use client::state::profile;
use crate::gui::window::{App, Profile};

/// Show the profile each time it loads
pub fn subscribe(app: &App, listeners: &mut Listeners) {
    let profile_app = app.clone_strong();
    listeners.subscribe(
        EventKind::ProfileLoaded,
        Box::new(move |_| show(&profile_app))
    );
}

fn show(app: &App) {
    if let Some(loaded) = profile::get() {
        let names = app.global::<Profile>();
        names.set_username(loaded.username.into());
        names.set_display_name(loaded.display_name.into());
    }
}
