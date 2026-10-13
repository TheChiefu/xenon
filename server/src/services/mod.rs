pub mod push;
pub mod xbox;

use std::sync::LazyLock;

use reqwest::Client;

/// Outbound HTTP client shared by every service
pub static CLIENT: LazyLock<Client> = LazyLock::new(Client::new);
