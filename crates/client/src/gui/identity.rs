//! Shows who is signed in on the toolbar's identity card

use slint::{ComponentHandle, SharedString};

use client::events;
use client::state::profile;
use crate::gui::subscriptions::Subscribe;
use crate::gui::window::{App, Profile};

// PUBLIC METHODS //

pub fn subscribe(app: &App) {
    events::PROFILE_CHANGED.subscribe_window(app, refresh);
}

// HELPER METHODS //

fn refresh(app: &App) {
    let displayed_profile = app.global::<Profile>();

    match profile::get() {
        Some(loaded) => {
            displayed_profile.set_username(loaded.username.into());
            displayed_profile.set_display_name(loaded.display_name.into());
        }
        None => {
            displayed_profile.set_username(SharedString::new());
            displayed_profile.set_display_name(SharedString::new());
        }
    }
}
