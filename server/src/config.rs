//! The config file, read once at startup

mod auth;
mod database;
mod files;
mod lengths;
mod network;
mod push;
mod responses;
mod server;
mod sockets;
mod steam;
mod xbox;

use std::sync::OnceLock;
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

pub use auth::Auth;
pub use database::Database;
pub use files::Files;
pub use lengths::Lengths;
pub use network::Network;
pub use push::Push;
pub use responses::Responses;
pub use server::Server;
pub use sockets::Sockets;
pub use xbox::Xbox;
pub use steam::Steam;

/// Static Global (configured once on load from file)
static CONFIG: OnceLock<Config> = OnceLock::new();

/// Config file layout this server reads.
const CONFIG_VERSION: u8 = 1;

/// Every setting the server reads at startup.
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub version: u8,
    pub server: Server,
    pub network: Network,
    pub files: Files,
    pub database: Database,
    pub auth: Auth,
    pub push: Option<Push>,
    pub xbox: Option<Xbox>,
    pub steam: Option<Steam>,
    pub lengths: Lengths,
    pub responses: Responses,
    pub sockets: Sockets,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            version: CONFIG_VERSION,
            server: Server::default(),
            network: Network::default(),
            files: Files::default(),
            database: Database::default(),
            auth: Auth::default(),
            push: None,
            xbox: None,
            steam: None,
            lengths: Lengths::default(),
            responses: Responses::default(),
            sockets: Sockets::default(),
        }
    }
}

/// Read the config file into the global static, writing defaults if it is absent.
/// Exit if the file cannot be parsed or fails validation
pub fn init(path: &str) {

    let config: Config = match std::fs::read_to_string(path) {
        Ok(text) => match toml::from_str(&text) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("{path}: {e}");
                std::process::exit(1);
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => write_default(path),
        Err(e) => {
            eprintln!("{path}: {e}");
            std::process::exit(1);
        },
    };

    if let Err(reason) = config.validate() {
        eprintln!("{path}: {reason}");
        std::process::exit(1);
    }

    // Set global in memory config
    CONFIG.set(config).expect("config already initialized");
}

/// Write the defaults to disk and returns them
fn write_default(path: &str) -> Config {
    let config = Config::default();

    match toml::to_string_pretty(&config) {
        Ok(text) => {
            if let Err(e) = std::fs::write(path, text) {
                eprintln!("could not write {path}: {e}");
            }
        }
        Err(e) => eprintln!("could not serialize default config: {e}"),
    }

    config
}

impl Config {

    /// Errors on the first invalid value in any section
    fn validate(&self) -> Result<(), String> {

        if self.version != CONFIG_VERSION {
            return Err(format!(
                "version {} is not supported, server is using version {}",
                self.version,
                CONFIG_VERSION
            ));
        }

        self.server.validate()?;
        self.network.validate()?;
        self.files.validate()?;
        self.database.validate()?;
        self.auth.validate()?;

        if let Some(push) = &self.push {
            push.validate()?;
        }

        if let Some(xbox) = &self.xbox {
            xbox.validate()?;

            if self.network.public_url.is_none() {
                return Err("network.public_url must be set when xbox is enabled".to_string());
            }
        }

        if let Some(steam) = &self.steam {
            steam.validate()?;

            if self.network.public_url.is_none() {
                return Err("network.public_url must be set when steam is enabled".to_string());
            }
        }

        self.lengths.validate()?;
        self.responses.validate()?;
        self.sockets.validate()?;

        Ok(())
    }

    /// The address the listener binds.
    /// Panics if the configured IP does not parse.
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(
            self.network.ip.parse().expect("network.ip validated at startup"),
            self.network.port
        )
    }

    /// Whether the listener binds a loopback address.
    pub fn binds_loopback(&self) -> bool {
        self.socket_addr().ip().is_loopback()
    }

    pub fn public_url(&self) -> &str {
        let url = self.network.public_url.as_deref().expect("network.public_url validated at startup");
        url.trim_end_matches('/')
    }

    /// Returns xbox section
    pub fn xbox(&self) -> &Xbox {
        self.xbox.as_ref().expect("xbox routes require xbox config")
    }

    /// Returns steam section
    pub fn steam(&self) -> &Steam {
        self.steam.as_ref().expect("steam routes require steam config")
    }

    /// Whether a certificate and key are configured for the listener.
    pub fn tls_configured(&self) -> bool {
        self.network.certificate.is_some() && self.network.key.is_some()
    }
}

/// Public accessor for static global config
/// Panics if called before the config is loaded
pub fn get() -> &'static Config {
    CONFIG.get().expect("config not initialized")
}
