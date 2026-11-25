//! Shows errors in the window's alert

use slint::{ComponentHandle, SharedString};

use client::api::ErrorMessage;
use client::events;
use crate::gui::subscriptions::{Subscribe, SubscribeWith};
use crate::gui::window::{App, Status};

// PUBLIC METHODS //

pub fn subscribe(app: &App) {
    events::ERROR.subscribe_window(app, show);
    events::WORKING.subscribe_window(app, clear);
}

// HELPER METHODS //

fn show(app: &App, message: ErrorMessage) {
    app.global::<Status>().set_error(message.into());
}

fn clear(app: &App) {
    app.global::<Status>().set_error(SharedString::new());
}
