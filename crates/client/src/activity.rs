//! Shows when the app is waiting on something

use slint::ComponentHandle;

use crate::events::{Event, Listeners};
use crate::AppWindow;

pub fn subscribe(window: &AppWindow, listeners: &mut Listeners) {
    let window = window.clone_strong();
    listeners.subscribe(Box::new(move |event| show(&window, event)));
}

fn show(window: &AppWindow, event: &Event) {
    match event {
        Event::Working => {
            window.set_busy(true);
        }
        Event::SignedIn(_) => {
            window.set_busy(false);
        }
        Event::Error(_) => {
            window.set_busy(false);
        }
    }
}
