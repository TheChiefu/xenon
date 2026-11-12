//! What the server sends a user's browser over their RTC socket, and when that socket closes

use std::sync::Arc;

use tokio::sync::{mpsc, watch};
use types::rtc::events::{RtcCloseCode, RtcServerEvent};

use crate::sockets;

pub struct Signal {
    outbound: mpsc::UnboundedSender<String>,
    closing: watch::Sender<Option<RtcCloseCode>>
}

impl Signal {

    /// A new signal, and the events its socket writes
    pub fn new() -> (Arc<Signal>, mpsc::UnboundedReceiver<String>) {
        let (outbound, receiver) = mpsc::unbounded_channel();
        let (closing, _) = watch::channel(None);

        (Arc::new(Signal { outbound, closing }), receiver)
    }

    /// Queue an event for the browser
    pub fn send(&self, event: &RtcServerEvent) {
        if let Some(text) = sockets::encode(event) {
            let _ = self.outbound.send(text);
        }
    }

    /// Ask for the socket to close, keeping the first code given
    pub fn end(&self, code: RtcCloseCode) {
        self.closing.send_if_modified(|current| {
            if current.is_some() {
                return false;
            }
            *current = Some(code);
            true
        });
    }

    /// Wait until the socket should close, returning the code to close with
    pub async fn ended(&self) -> RtcCloseCode {
        let mut receiver = self.closing.subscribe();

        match receiver.wait_for(Option::is_some).await {
            Ok(code) => code.unwrap_or(RtcCloseCode::Normal),
            Err(_) => RtcCloseCode::Normal
        }
    }
}
