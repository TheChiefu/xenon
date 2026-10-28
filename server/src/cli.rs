//! Commands for server
//!
//! Run the server inside `screen` to deatch from user console:
//! - Start server:         `screen -S xenon ./xenon`
//! - Detach from screen:   Ctrl+A then D
//! - Reattach to screen:   `screen -r xenon`
//! 

mod files;
mod ip;
mod user;

use std::io::Write;
use std::sync::LazyLock;

use tokio::sync::{mpsc, Notify};

use crate::error::{AppError, Result};
use crate::state::AppState;

/// Signal the `exit` command, shutting the server down
pub static EXIT: LazyLock<Notify> = LazyLock::new(Notify::new);

/// Signal that the server is listening
pub static READY: LazyLock<Notify> = LazyLock::new(Notify::new);


pub const COMMANDS: &[&str] = &[
    "user <command>",
    "files <command>",
    "ip <command>",
    "stop",
    "exit",
];

/// Reads commands typed into the terminal and runs each one
pub async fn console(state: AppState) {
    let (sender, mut lines) = mpsc::unbounded_channel();

    // Read stdin on a separate thread, sending each line to the loop below
    std::thread::spawn(move || {
        for line in std::io::stdin().lines() {
            let Ok(line) = line else { return };
            if sender.send(line).is_err() {
                return;
            }
        }
    });

    READY.notified().await;

    loop {
        print!("> ");
        let _ = std::io::stdout().flush();

        // Stop reading once the terminal is closed
        let Some(line) = lines.recv().await else { return };

        let args: Vec<&str> = line.split_whitespace().collect();

        // Signal shutdown and stop reading commands
        if let ["stop"] | ["exit"] = args[..] {
            EXIT.notify_one();
            return;
        }

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
        ["files", args_s @ ..] => files::run(state, args_s).await,
        ["ip", args_s @ ..] => ip::run(state, args_s).await,
        _ => Err(usage(&[COMMANDS].concat())),
    }
}

// Helper Methods //

/// Lists the given commands as a usage error
fn usage(commands: &[&str]) -> AppError {
    AppError::Validation(format!(
            "usage: {}",
            commands.join("\n       ")
    ))
}
