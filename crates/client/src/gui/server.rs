//! Shows the connected server on the toolbar's server card

use slint::{ComponentHandle, SharedString};
use types::bytesize::ByteSize;

use client::events;
use client::state::server;
use crate::gui::subscriptions::Subscribe;
use crate::gui::window::{App, Server};

// PUBLIC METHODS //

pub fn subscribe(app: &App) {
    events::SERVER_CHANGED.subscribe_window(app, refresh);
}

// HELPER METHODS //

fn refresh(app: &App) {
    let displayed = app.global::<Server>();

    match server::get() {
        Some(info) => {
            displayed.set_name(info.name.into());
            displayed.set_description(info.description.into());
            displayed.set_kind(info.kind.into());
            displayed.set_version(info.version.into());
            displayed.set_file_size_max(
                ByteSize::from_int(info.limit_file_bytes_max)
                .to_string()
                .into()
            );
            displayed.set_attachments_max(info.limit_attachments_per_message_max as i32);
        }
        None => {
            displayed.set_name(SharedString::new());
            displayed.set_description(SharedString::new());
            displayed.set_kind(SharedString::new());
            displayed.set_version(SharedString::new());
            displayed.set_file_size_max(SharedString::new());
            displayed.set_attachments_max(0);
        }
    }
}
