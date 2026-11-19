//! Starts the desktop client
mod gui;

fn main() -> Result<(), slint::PlatformError> {
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime should start");

    // Lets tokio::spawn be called from the UI thread while this is in scope
    let _runtime_context = runtime.enter();

    gui::run()
}
