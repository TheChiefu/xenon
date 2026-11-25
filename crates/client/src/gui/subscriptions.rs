//! Subscribing window methods to client events

use slint::{ComponentHandle, Weak};

use client::events::{Event, EventWith};

/// Subscribes window methods to events
pub trait Subscribe {

    /// Add a handler that runs the method on the window
    fn subscribe_window<W>(&self, window: &W, method: fn(&W))
    where
        W: ComponentHandle + 'static; // Any Slint window
}

impl Subscribe for Event {
    fn subscribe_window<W>(&self, window: &W, method: fn(&W))
    where
        W: ComponentHandle + 'static,
    {
        let ptr = window.as_weak();
        self.subscribe(move || run_on_window(&ptr, method));
    }
}

/// Subscribes window methods to events with data
pub trait SubscribeWith<T> {

    /// Add a handler that runs the method on the window with the event's data
    fn subscribe_window<W>(&self, window: &W, method: fn(&W, T))
    where
        W: ComponentHandle + 'static; // Any Slint window
}

impl<T> SubscribeWith<T> for EventWith<T>
where
    T: Clone + Send + 'static, // Event data is copied and moved to the UI thread
{
    fn subscribe_window<W>(&self, window: &W, method: fn(&W, T))
    where
        W: ComponentHandle + 'static,
    {
        let ptr = window.as_weak();
        self.subscribe(move |data: &T| {
            let data = data.clone();
            run_on_window(&ptr, move |window| method(window, data));
        });
    }
}

// HELPER METHODS //

/// Run on the window's UI thread while the window is open
fn run_on_window<W, R>(ptr: &Weak<W>, run: R)
where
    W: ComponentHandle + 'static,
    R: FnOnce(&W) + Send + 'static, // Moved to the UI thread and run once
{
    let target = ptr.clone();

    let run_method = move || {
        let Some(window) = target.upgrade() else {
            return;
        };
        run(&window);
    };

    // Run on the UI thread
    let _ = slint::invoke_from_event_loop(run_method);
}
