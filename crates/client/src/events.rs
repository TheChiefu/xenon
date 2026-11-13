//! Events published by one part of the client and handled by others

use std::sync::OnceLock;

use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::errors::ErrorMessage;
use crate::session::Session;

pub enum Event {
    Working,
    SignedIn(Session),
    Error(ErrorMessage),
}

static SENDER: OnceLock<UnboundedSender<Event>> = OnceLock::new();

/// Publishes an event to every listener, from any thread
pub fn dispatch(event: Event) {
    if let Some(sender) = SENDER.get() {
        // Only fails once the app is closing and nothing is listening anymore
        let _ = sender.send(event);
    }
}

pub struct Listeners {
    list: Vec<Box<dyn Fn(&Event)>>,
}

impl Listeners {
    pub fn new() -> Self {
        Listeners { list: Vec::new() }
    }

    pub fn subscribe(&mut self, listener: Box<dyn Fn(&Event)>) {
        self.list.push(listener);
    }

    /// Delivers every dispatched event to the listeners, on the UI thread
    pub fn start(self) {
        let (sender, mut receiver) = unbounded_channel();
        SENDER.set(sender).expect("events should only start once");

        slint::spawn_local(async move {
            while let Some(event) = receiver.recv().await {
                for listener in &self.list {
                    listener(&event);
                }
            }
        })
        .expect("events should start on the UI thread");
    }
}
