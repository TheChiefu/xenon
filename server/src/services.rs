pub mod push;
pub mod steam;
pub mod xbox;

use std::sync::LazyLock;
use std::time::Duration;

use reqwest::Client;

use crate::config;

/// Outbound HTTP client shared by every service
pub static CLIENT: LazyLock<Client> = LazyLock::new(build_client);

/// Build outbound HTTP client at startup when config enables a service
pub fn init() {
    let config = config::get();

    if config.xbox.is_none()
        && config.push.is_none()
        && config.steam.is_none()
    {
        return;
    }

    LazyLock::force(&CLIENT);
}

// Helper Methods //

/// Build outbound HTTP client with request timeout, exits server if it cannot be built
fn build_client() -> Client {
    let timeout = Duration::from_secs(config::get().services.request_timeout_seconds);

    let built = Client::builder()
        .timeout(timeout)
        .build();

    match built {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("a service is enabled in config, but the outbound HTTP client could not be created: {e}");
            std::process::exit(1);
        }
    }
}
