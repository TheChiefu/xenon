//! Events published by one part of the client and handled by others

use std::collections::HashMap;
use std::sync::OnceLock;

use tokio::sync::mpsc::{self, UnboundedSender};

use crate::api::ErrorMessage;

static SENDER: OnceLock<UnboundedSender<Event>> = OnceLock::new();

// TYPES //
pub enum Event {
    Working,
    Done,
    Error(ErrorMessage),
    ProfileLoaded,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    Working,
    Done,
    Error,
    ProfileLoaded,
}

// METHODS // 
impl Event {
    pub fn kind(&self) -> EventKind {
        match self {
            Event::Working => EventKind::Working,
            Event::Done => EventKind::Done,
            Event::Error(_) => EventKind::Error,
            Event::ProfileLoaded => EventKind::ProfileLoaded,
        }
    }
}
pub struct Listeners {
    by_kind: HashMap<EventKind, Vec<Box<dyn Fn(&Event)>>>,
}

impl Listeners {
    pub fn new() -> Self {
        Listeners { by_kind: HashMap::new() }
    }

    /// Register a listener for one kind of event
    pub fn subscribe(&mut self, kind: EventKind, listener: Box<dyn Fn(&Event)>) {
        self.by_kind.entry(kind).or_default().push(listener);
    }

    /// Deliver each dispatched event to the listeners of its kind, on the UI thread
    pub fn start(self) {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        SENDER.set(sender).expect("events should only start once");

        slint::spawn_local(async move {
            while let Some(event) = receiver.recv().await {
                if let Some(listeners) = self.by_kind.get(&event.kind()) {
                    for listener in listeners {
                        listener(&event);
                    }
                }
            }
        })
        .expect("events should start on the UI thread");
    }
}

/// Publish an event to its listeners, from any thread
pub fn dispatch(event: Event) {
    if let Some(sender) = SENDER.get() {
        // Only fails once the app is closing and nothing is listening anymore
        let _ = sender.send(event);
    }
}