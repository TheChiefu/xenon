//! Events published by one part of the client and subscribed to by others

use std::sync::Mutex;

use crate::api::ErrorMessage;

const LOCK_FAILED: &str = "handlers should not be left locked by a crash";


/// Event with no data
pub struct Event {
    handlers: Mutex<Vec<Box<dyn Fn() + Send + Sync>>> // Vec of function signatures
}
impl Event {

    /// Create an event with no handlers
    const fn new() -> Self {
        Event {
            handlers: Mutex::new(Vec::new())
        }
    }

    /// Add a handler to the event
    pub fn subscribe<H>(&self, handler: H)
    where
        H: Fn() + Send + Sync + 'static, // Callable from any thread
    {
        self.handlers.lock().expect(LOCK_FAILED).push(Box::new(handler));
    }

    /// Announce the event
    pub fn publish(&self) {
        let handlers = self.handlers.lock().expect(LOCK_FAILED);
        for handler in handlers.iter() {
            handler();
        }
    }
}

/// Event with data
pub struct EventWith<T> {
    handlers: Mutex<Vec<Box<dyn Fn(&T) + Send + Sync>>> // Vec of function signatures (w/data)
}
impl<T> EventWith<T> {

    /// Create an event with no handlers
    const fn new() -> Self {
        EventWith {
            handlers: Mutex::new(Vec::new())
        }
    }

    /// Add a handler to the event
    pub fn subscribe<H>(&self, handler: H)
    where
        H: Fn(&T) + Send + Sync + 'static, // Callable with the event's data from any thread
    {
        self.handlers.lock().expect(LOCK_FAILED).push(Box::new(handler));
    }

    /// Announce the event
    pub fn publish(&self, data: T) {
        let handlers = self.handlers.lock().expect(LOCK_FAILED);
        for handler in handlers.iter() {
            handler(&data);
        }
    }
}


// EVENTS //

pub static WORKING: Event = Event::new();
pub static DONE: Event = Event::new();
pub static PROFILE_CHANGED: Event = Event::new();
pub static SIGNED_IN: Event = Event::new();
pub static SIGNED_OUT: Event = Event::new();
pub static ERROR: EventWith<ErrorMessage> = EventWith::new();
