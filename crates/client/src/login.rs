//! Signing in from the login screen

use slint::SharedString;

use crate::api;
use crate::events::{self, Event};

pub fn sign_in(username: SharedString, password: SharedString, server: SharedString) {
    events::dispatch(Event::Working);
    tokio::spawn(request(username, password, server));
}

async fn request(username: SharedString, password: SharedString, server: SharedString) {
    match api::login(server, username, password).await {
        Ok(session) => events::dispatch(Event::SignedIn(session)),
        Err(reason) => events::dispatch(Event::Error(reason)),
    }
}
