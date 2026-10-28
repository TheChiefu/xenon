//! The config file, read once at startup

mod bind;
mod calls;
mod database;
mod info;
mod lengths;
mod logging;
mod push;
mod rate_limit;
mod responses;
mod services;
mod session;
mod sockets;
mod steam;
mod storage;
mod uploads;
mod xbox;

use std::sync::OnceLock;
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

pub use bind::Bind;
pub use calls::Calls;
pub use database::Database;
pub use info::Info;
pub use lengths::Lengths;
pub use logging::Logging;
pub use push::Push;
pub use rate_limit::RateLimit;
pub use responses::Responses;
pub use services::Services;
pub use session::Session;
pub use sockets::Sockets;
pub use storage::Storage;
pub use uploads::Uploads;
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
    pub info: Info,
    pub bind: Bind,
    pub storage: Storage,
    pub database: Database,
    pub logging: Logging,
    pub session: Session,
    pub push: Option<Push>,
    pub xbox: Option<Xbox>,
    pub steam: Option<Steam>,
    pub lengths: Lengths,
    pub uploads: Uploads,
    pub responses: Responses,
    pub sockets: Sockets,
    pub calls: Calls,
    pub services: Services,
    pub rate_limit: RateLimit,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            version: CONFIG_VERSION,
            info: Info::default(),
            bind: Bind::default(),
            storage: Storage::default(),
            database: Database::default(),
            logging: Logging::default(),
            session: Session::default(),
            push: None,
            xbox: None,
            steam: None,
            lengths: Lengths::default(),
            uploads: Uploads::default(),
            responses: Responses::default(),
            sockets: Sockets::default(),
            calls: Calls::default(),
            services: Services::default(),
            rate_limit: RateLimit::default(),
        }
    }
}

/// Reads the config file into the global static, writing defaults if it is absent.
///
/// Exits the process if the file cannot be parsed or fails validation.
///
/// # Arguments
///
/// * `path` - Path to the config toml file.
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

/// Writes the defaults to disk and returns them.
///
/// # Arguments
///
/// * `path` - Path the config toml file is written to.
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

        self.bind.validate()?;
        self.storage.validate()?;
        self.database.validate()?;
        self.session.validate()?;

        if let Some(push) = &self.push {
            push.validate()?;
        }

        if let Some(xbox) = &self.xbox {
            xbox.validate()?;

            if self.bind.public_url.is_none() {
                return Err("bind.public_url must be set when xbox is enabled".to_string());
            }
        }

        if let Some(steam) = &self.steam {
            steam.validate()?;

            if self.bind.public_url.is_none() {
                return Err("bind.public_url must be set when steam is enabled".to_string());
            }
        }

        self.lengths.validate()?;
        self.uploads.validate()?;
        self.responses.validate()?;
        self.services.validate()?;
        self.sockets.validate()?;
        self.calls.validate()?;
        self.rate_limit.validate()?;
        self.logging.validate()?;

        Ok(())
    }

    /// The address the listener binds.
    /// Panics if `bind.ip` does not parse.
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(
            self.bind.ip.parse().expect("bind.ip validated at startup"),
            self.bind.port
        )
    }

    /// Whether the listener binds a loopback address.
    pub fn binds_loopback(&self) -> bool {
        self.socket_addr().ip().is_loopback()
    }

    pub fn public_url(&self) -> &str {
        let url = self.bind.public_url.as_deref().expect("bind.public_url validated at startup");
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
        self.bind.certificate.is_some() && self.bind.key.is_some()
    }
}

/// Public accessor for static global config
///
/// # Panics
///
/// Panics if called before `init`.
pub fn get() -> &'static Config {
    CONFIG.get().expect("config not initialized")
}
