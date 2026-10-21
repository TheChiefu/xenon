mod api;
mod bytesize;
mod cli;
mod config;
mod db;
mod error;
mod ratelimit;
mod routes;
mod serve;
mod services;
mod shared;
mod sockets;
mod state;
mod utils;
mod validate;

use std::path::Path;
use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use uuid::Uuid;

use crate::error::Result;
use crate::shared::GlobalRole;
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
    ensure_files();

    let pool = connect_database().await?;

    // Ensure bootstrap account is created with admin access
    ensure_account(&pool).await?;

    // Ensure VAPID keys exist for browsers to subscribe against
    services::push::send::ensure_keypair(&pool).await?;

    // Build outbound HTTP client if any service is enabled
    services::init();

    // Server server
    let app_state = AppState::new(pool);

    // Read blocked addresses from the database
    load_blocked_addresses(&app_state).await?;

    // Read commands typed into the terminal
    tokio::spawn(cli::console(app_state.clone()));

    services::xbox::start(app_state.clone());

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

    // Create initial DB if it does not already exists
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

/// Creates the bootstrap account, printing its generated password once.
///
/// # Arguments
///
/// * `pool` - Pool of SQL connections.
///
/// # Errors
///
/// Returns `AppError::Hash` if the password cannot be hashed, and
async fn ensure_account(pool: &SqlitePool) -> Result<()> {
    let password = utils::generate_code(12); // Fixed, one time password
    let hash = utils::hash_password(&password)?;
    let username = "admin";
    let display_name = "Admin";
    let id = Uuid::now_v7();

    // Check if any account exists on the server, if none do create one
    let mut conn = pool.acquire().await?;
    let count = db::get_user_count(&mut conn).await?;

    // Skip if accounts already exist
    if count > 0 {
        return Ok(());
    }

    // Attempt to create account
    let mut conn = pool.acquire().await?;
    let result = db::insert_user(
        &mut conn,
        id,
        username,
        display_name,
        &hash,
        GlobalRole::Admin
    ).await;

    match result {
        Ok(()) => {
            // Printed, never logged. This is the only time the password is shown
            println!("+-------- Admin account created --------+");
            println!("           Username:\t{username}         ");
            println!("           Password:\t{password}         ");
            println!("  Save credentials (this is NOT logged)  ");
            println!("+---------------------------------------+");
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Sets where tracing output is written.
///
/// Output goes to stdout, and is appended to `storage.log` if one is set.
fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new(
                format!("xenon={},sqlx=warn", config::get().logging.level)
            )
        });

    let path = &config::get().logging.file;
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

/// Creates the file storage directories and verifies the temp one is writable.
///
/// Exits the process if any step fails.
fn ensure_files() {

    // Create or check if files folder path exists
    let files_path = &config::get().storage.files;
    if let Err(e) = std::fs::create_dir_all(files_path) {
        eprintln!("{files_path}: {e}");
        std::process::exit(1);
    }

    // Create/check "tmp" folder where streaming uploads sit
    let subdir = Path::new(files_path).join("tmp");
    if let Err(e) = std::fs::create_dir_all(&subdir) {
        eprintln!("{}: {e}", subdir.display());
        std::process::exit(1);
    }

    // Test if tmp directory is writable (create probe file)
    let test_file = subdir.join("test");
    if let Err(e) = std::fs::write(&test_file, "") {
        eprintln!("{} is not writable: {e}", subdir.display());
        std::process::exit(1);
    }

    // Remove probe file now that the write succeeded
    if let Err(e) = std::fs::remove_file(&test_file) {
        eprintln!("{}: {e}", test_file.display());
        std::process::exit(1);
    }

    tracing::info!("file storage ready at: {files_path}");

}

// Shutdown //

/// Resolves on SIGTERM.
#[cfg(unix)]
async fn terminate() {
    use tokio::signal::unix::{signal, SignalKind};
    signal(SignalKind::terminate())
        .expect("failed to install SIGTERM handler")
        .recv()
        .await;
    tracing::info!("Received \"SIGTERM\", terminating program...");
}

#[cfg(not(unix))]
async fn terminate() {
    std::future::pending::<()>().await;
}

/// Resolves on the first shutdown signal.
///
/// Axum polls this alongside serving.
async fn shutdown_signal() {
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = terminate() => {},
        _ = cli::EXIT.notified() => {},
    }
    tracing::info!("Shutting down server");
}
