//! Shows when the app is waiting on something

use slint::ComponentHandle;

use client::events::{EventKind, Listeners};
use crate::gui::window::{App, Status};

/// Show the progress bar while work runs, and hide it when the work finishes or fails
pub fn subscribe(app: &App, listeners: &mut Listeners) {
    let working_app = app.clone_strong();
    listeners.subscribe(
        EventKind::Working,
        Box::new(move |_| working_app.global::<Status>().set_busy(true))
    );

    let done_app = app.clone_strong();
    listeners.subscribe(
        EventKind::Done,
        Box::new(move |_| done_app.global::<Status>().set_busy(false))
    );

    let error_app = app.clone_strong();
    listeners.subscribe(
        EventKind::Error,
        Box::new(move |_| error_app.global::<Status>().set_busy(false))
    );
}
