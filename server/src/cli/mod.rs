//! Commands for server
//!
//! Run the server inside `screen` to deatch from user console:
//! - Start server:         `screen -S xenon ./xenon`
//! - Detach from screen:   Ctrl+A then D
//! - Reattach to screen:   `screen -r xenon`
//! 

mod ip;
mod user;

use std::io::Write;
use std::sync::LazyLock;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Notify;

use crate::error::{AppError, Result};
use crate::state::AppState;

/// Signal the `exit` command, shutting the server down
pub static EXIT: LazyLock<Notify> = LazyLock::new(Notify::new);

/// Signal that the server is listening
pub static READY: LazyLock<Notify> = LazyLock::new(Notify::new);


pub const COMMANDS: &[&str] = &[
    "user <command>",
    "ip <command>",
    "stop",
    "exit",
];

/// Reads commands typed into the terminal and runs each one
pub async fn console(state: AppState) {
    let mut lines = BufReader::new(tokio::io::stdin()).lines();

    READY.notified().await;

    loop {
        print!("> ");
        let _ = std::io::stdout().flush();

        // Stop reading once the terminal is closed
        let line = match lines.next_line().await {
            Ok(Some(line)) => line,
            _ => return,
        };

        let args: Vec<&str> = line.split_whitespace().collect();

        match run(&state, &args).await {
            Ok(()) => {}
            Err(e) => eprintln!("{e}"),
        }
    }
}

/// Runs the command named by the arguments
async fn run(state: &AppState, args: &[&str]) -> Result<()> {

    // Exact full match or match + slice of remaining arguments passed along
    match args {
        [] => Ok(()),
        ["user", args_s @ ..] => user::run(state, args_s).await,
        ["ip", args_s @ ..] => ip::run(state, args_s).await,
        ["stop"] | ["exit"] => shutdown(),
        _ => Err(usage(&[COMMANDS].concat())),
    }
}

/// Shuts the server down
fn shutdown() -> Result<()> {
    EXIT.notify_one();
    Ok(())
}

/// Lists the given commands as a usage error
fn usage(commands: &[&str]) -> AppError {
    AppError::Validation(format!(
            "usage: {}",
            commands.join("\n       ")
    ))
}
