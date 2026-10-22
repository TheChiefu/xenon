//! What every handler is given:
//! - the database
//! - who is connected

use axum::extract::FromRef;
use sqlx::SqlitePool;

use crate::ratelimit::{AddressLimits, LoginFailures};
use crate::sockets::call::registry::ActiveCalls;
use crate::sockets::chat::game_presence::ReportedGames;
use crate::sockets::chat::registry::Registry;

/// Cloned per request
///
/// `FromRef` generates a per-field accessor
/// letting handlers keep asking for `State<SqlitePool>`
#[derive(Clone, FromRef)]
pub struct AppState {
    pub pool: SqlitePool,
    pub registry: Registry,

    /// Who is in each room's call
    pub calls: ActiveCalls,

    /// What each user's linked account last reported
    pub game_presence: ReportedGames,

    /// Failures each IP has made, and whether it is blocked
    pub address_limits: AddressLimits,

    /// Failed logins each account has made since its last successful login
    pub login_failures: LoginFailures
}

impl AppState {

    /// Builds the state a router is handed.
    ///
    /// # Arguments
    ///
    /// * `pool` - Pool of SQL connections.
    pub fn new(pool: SqlitePool) -> Self {
        AppState {
            pool,
            registry: Registry::default(),
            calls: ActiveCalls::default(),
            game_presence: ReportedGames::default(),
            address_limits: AddressLimits::default(),
            login_failures: LoginFailures::default(),
        }
    }
}
