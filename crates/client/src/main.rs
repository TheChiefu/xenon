//! Starts the desktop client

mod activity;
mod api;
mod errors;
mod events;
mod login;
mod session;

use events::Listeners;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime should start");

    // Lets tokio::spawn be called from the UI thread while this is in scope
    let _runtime_context = runtime.enter();

    let window = AppWindow::new()?;

    let mut listeners = Listeners::new();
    activity::subscribe(&window, &mut listeners);
    errors::subscribe(&window, &mut listeners);
    listeners.start();

    window.on_sign_in(login::sign_in);
    window.run()
}
