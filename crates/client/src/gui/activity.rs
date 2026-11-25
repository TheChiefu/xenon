//! Shows when the app is waiting on something

use slint::ComponentHandle;

use client::api::ErrorMessage;
use client::events;
use crate::gui::subscriptions::{Subscribe, SubscribeWith};
use crate::gui::window::{App, Status};

// PUBLIC METHODS //

pub fn subscribe(app: &App) {
    events::WORKING.subscribe_window(app, show_busy);
    events::DONE.subscribe_window(app, hide_busy);
    events::ERROR.subscribe_window(app, hide_busy_after_error);
}

// HELPER METHODS //

fn show_busy(app: &App) {
    app.global::<Status>().set_busy(true);
}

fn hide_busy(app: &App) {
    app.global::<Status>().set_busy(false);
}

fn hide_busy_after_error(app: &App, _: ErrorMessage) {
    app.global::<Status>().set_busy(false);
}
