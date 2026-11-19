//! Shows errors in the window's alert

use slint::{ComponentHandle, SharedString};

use client::events::{Event, EventKind, Listeners};
use crate::gui::window::{App, Status};

/// Show each error, and clear it when new work starts
pub fn subscribe(app: &App, listeners: &mut Listeners) {
    let error_app = app.clone_strong();
    listeners.subscribe(
        EventKind::Error,
        Box::new(move |event| show(&error_app, event))
    );

    let working_app = app.clone_strong();
    listeners.subscribe(
        EventKind::Working,
        Box::new(move |_| clear(&working_app))
    );
}

fn show(app: &App, event: &Event) {
    if let Event::Error(message) = event {
        app.global::<Status>().set_error(message.as_str().into());
    }
}

fn clear(app: &App) {
    app.global::<Status>().set_error(SharedString::new());
}
