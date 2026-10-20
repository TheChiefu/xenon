//! Account commands run against the database without starting the server.

use sqlx::SqlitePool;

use crate::error::Result;
use crate::utils;

const USAGE: &str = "usage: xenon user unlock <username>\n       xenon user reset <username>";

/// Runs the command named by the arguments, exiting the process if it is unknown
pub async fn run(pool: &SqlitePool, args: &[String]) -> Result<()> {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();

    match args.as_slice() {
        ["user", "unlock", username] => unlock(pool, username).await,
        ["user", "reset", username] => reset(pool, username).await,
        _ => {
            eprintln!("{USAGE}");
            std::process::exit(2);
        }
    }
}

/// Lets a locked account log in again
async fn unlock(pool: &SqlitePool, username: &str) -> Result<()> {
    let updated = sqlx::query("UPDATE users SET locked = 0 WHERE username = ?1 AND deleted_at IS NULL")
        .bind(username)
        .execute(pool)
        .await?
        .rows_affected();

    if updated == 0 {
        eprintln!("no account named \"{username}\"");
        std::process::exit(1);
    }

    println!("unlocked \"{username}\"");
    Ok(())
}

/// Replaces an account's password with a generated one, printing it once
async fn reset(pool: &SqlitePool, username: &str) -> Result<()> {
    let password = utils::generate_code(12);
    let hash = utils::hash_password(&password)?;

    let updated = sqlx::query("UPDATE users SET password_hash = ?1 WHERE username = ?2 AND deleted_at IS NULL")
        .bind(hash)
        .bind(username)
        .execute(pool)
        .await?
        .rows_affected();

    if updated == 0 {
        eprintln!("no account named \"{username}\"");
        std::process::exit(1);
    }

    // Printed, never logged
    println!("new password for \"{username}\": {password}");
    Ok(())
}
