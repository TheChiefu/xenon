//! What every handler is given:
//! - the database
//! - who is connected

use axum::extract::FromRef;
use sqlx::SqlitePool;

use crate::ratelimit::{LoginFailures, RegisterLimit};
use crate::sockets::game_presence::ReportedGames;
use crate::sockets::registry::Registry;

/// Cloned per request
///
/// `FromRef` generates a per-field accessor
/// letting handlers keep asking for `State<SqlitePool>`
#[derive(Clone, FromRef)]
pub struct AppState {
    pub pool: SqlitePool,
    pub registry: Registry,

    /// What each user's linked account last reported
    pub game_presence: ReportedGames,

    /// Registration attempts each IP has made
    pub register_limit: RegisterLimit,

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
            game_presence: ReportedGames::default(),
            register_limit: RegisterLimit::default(),
            login_failures: LoginFailures::default(),
        }
    }
}
