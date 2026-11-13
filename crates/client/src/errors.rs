//! Shows errors in the window's error bar

use slint::{ComponentHandle, SharedString};

use crate::events::{Event, Listeners};
use crate::AppWindow;

pub type ErrorMessage = String;

pub fn subscribe(window: &AppWindow, listeners: &mut Listeners) {
    let window = window.clone_strong();
    listeners.subscribe(Box::new(move |event| show(&window, event)));
}

fn show(window: &AppWindow, event: &Event) {
    match event {
        Event::Working => {
            window.set_error(SharedString::new());
        }
        Event::SignedIn(_) => {
            window.set_error(SharedString::new());
        }
        Event::Error(reason) => {
            window.set_error(reason.as_str().into());
        }
    }
}
