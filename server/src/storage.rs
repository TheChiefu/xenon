//! On-disk layout of file storage

use std::path::{Path, PathBuf};

use crate::config;
use crate::error::Result;

// PUBLIC METHODS //

/// Creates the file storage directories and verifies the temp one is writable.
///
/// Exits the process if any step fails.
pub fn ensure() {

    // Create or check if files folder path exists
    let files_path = &config::get().files.path;
    if let Err(e) = std::fs::create_dir_all(files_path) {
        eprintln!("{files_path}: {e}");
        std::process::exit(1);
    }

    // Create/check folder where streaming uploads sit
    let subdir = tmp_path();
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

/// Folder where uploads are written until hashed
pub fn tmp_path() -> PathBuf {
    Path::new(&config::get().files.path).join("tmp")
}

/// Maps a hash to the file holding its bytes, such as `files/ab/cd/abcd...`
pub fn blob_path(hex: &str) -> PathBuf {
    create_shard_path(hex).join(hex)
}

/// Create shard directory and rename the temp file into it
pub async fn move_to_shard(tmp_path: &Path, blob_path: &Path) -> Result<()> {
    if let Some(shard_dir) = blob_path.parent() {
        tokio::fs::create_dir_all(shard_dir).await?;
    }
    tokio::fs::rename(tmp_path, blob_path).await?;
    Ok(())
}

// HELPER METHODS //

/// Maps a hash to the directory holding it, such as `files/ab/cd`
fn create_shard_path(hex: &str) -> PathBuf {
    Path::new(&config::get().files.path)
        .join(&hex[0..2]) // Top Level first 2 hex chars
        .join(&hex[2..4]) // Second Level second 2 hex chars
}
