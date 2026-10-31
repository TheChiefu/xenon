//! Starts the server: config, database, services, then the HTTP router

mod api;
mod cli;
mod config;
mod db;
mod error;
mod ratelimit;
mod routes;
mod security;
mod serve;
mod services;
mod sockets;
mod state;
mod storage;
mod types;
mod utils;
mod validate;

use std::time::Duration;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use tracing_subscriber::fmt::writer::MakeWriterExt;

use crate::error::Result;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<()> {

    let config_path = std::env::var("XENON_CONFIG").unwrap_or_else(|_| "config.toml".into());

    // Read configuration file (create defaults if not existing)
    config::init(&config_path);

    // Setup tracing
    init_tracing();
    tracing::info!("loaded configuration file: {config_path}");
    tracing::info!("starting server");

    // Check if file directory is setup
    storage::ensure();

    let pool = connect_database().await?;

    // Tell the owner when no account exists yet
    let mut conn = pool.acquire().await?;
    if db::user_count(&mut conn).await? == 0 {
        tracing::warn!("no accounts exist, please create one [> help]");
    }
    drop(conn);

    // Build outbound HTTP client if any service is enabled
    services::init();

    // Server state
    let app_state = AppState::new(pool);

    // Read blocked addresses from the database
    load_blocked_addresses(&app_state).await?;

    // Read commands typed into the terminal
    tokio::spawn(cli::console(app_state.clone()));

    services::start(&app_state).await?;

    let app: axum::Router = routes::router(app_state);
    let bind_addr = config::get().socket_addr();

    if config::get().tls_configured() {
        serve::tls(app, bind_addr).await;
    } else {
        serve::plaintext(app, bind_addr).await;
    }

    Ok(())

}

/// Opens the database, creating it and its tables if they do not exist
async fn connect_database() -> Result<SqlitePool> {

    // Set DB options and properties
    let database = &config::get().database;
    let db_path = &database.path;
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal) // Write-Ahead Logging
        .synchronous(SqliteSynchronous::Full)
        .busy_timeout(Duration::from_secs(database.busy_timeout_seconds))
        .pragma("secure_delete", "ON");

    let pool = SqlitePoolOptions::new()
        .max_connections(database.max_connections)
        .min_connections(database.min_connections)
        .acquire_timeout(Duration::from_secs(database.acquire_timeout_seconds))
        .idle_timeout(database.idle_timeout())
        .connect_with(options)
        .await?;

    // Apply any migrations not yet run
    sqlx::migrate!().run(&pool).await?;
    tracing::info!("connected to database: {db_path}");

    Ok(pool)
}

/// Marks every address saved in the database as blocked
async fn load_blocked_addresses(app_state: &AppState) -> Result<()> {
    let mut conn = app_state.pool.acquire().await?;
    let blocked = db::blocked_addresses(&mut conn).await?;
    app_state.address_limits.load(blocked);
    Ok(())
}

/// Sets where tracing output is written.
///
/// Output goes to stdout, and is appended to the log file if one is set.
fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new(
                format!("xenon={},sqlx=warn", config::get().server.log_level)
            )
        });

    let path = &config::get().server.log_file;
    if path.is_empty() {
        tracing_subscriber::fmt().with_env_filter(filter).init();
        return;
    }

    let file = match std::fs::OpenOptions::new().create(true).append(true).open(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("{path}: {e}");
            std::process::exit(1);
        }
    };

    // Color codes are written literally into a file, so they are left off
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(file.and(std::io::stdout))
        .with_ansi(false)
        .init();
}
