//! `files` commands, removing stored files nothing uses.

use std::num::NonZeroUsize;

use sqlx::SqlitePool;

use crate::api;
use crate::error::{AppError, Result};
use crate::state::AppState;

pub const COMMANDS: &[&str] = &[
    "files clean",
    "files clean -t <threads>",
];

/// Runs the `files` command named by the arguments
pub async fn run(state: &AppState, args: &[&str]) -> Result<()> {
    match args {
        ["clean"] => clean(&state.pool, None).await,
        ["clean", "-t", threads] => clean(&state.pool, Some(threads)).await,
        _ => Err(super::usage(COMMANDS)),
    }
}

/// Deletes stored files nothing uses
async fn clean(pool: &SqlitePool, threads: Option<&str>) -> Result<()> {
    let threads = thread_count(threads)?;
    let unused_files = api::files::delete_unused_rows(pool).await?;
    api::files::delete_unused_files(&unused_files, threads).await?;

    println!("removed {} unused files", unused_files.len());
    Ok(())
}

// Helper Methods //

/// Thread count typed with `-t`, or every available thread when none is given
fn thread_count(threads: Option<&str>) -> Result<usize> {

    let Some(threads) = threads else {
        return Ok(std::thread::available_parallelism()?.get());
    };

    match threads.parse::<NonZeroUsize>() {
        Ok(count) => Ok(count.get()),
        Err(_) => Err(AppError::Validation(format!("\"{threads}\" is not a valid thread count"))),
    }
}
