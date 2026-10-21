//! Account commands run against the database without starting the server.

use std::net::IpAddr;

use sqlx::SqlitePool;

use crate::db;
use crate::error::Result;
use crate::utils;

const USAGE: &str = "\
usage: xenon user unlock <username>
       xenon user reset <username>
       xenon ip list
       xenon ip block <address>
       xenon ip unblock <address>";

/// Runs the command named by the arguments, exiting the process if it is unknown
pub async fn run(pool: &SqlitePool, args: &[String]) -> Result<()> {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();

    match args.as_slice() {
        ["user", "unlock", username] => unlock(pool, username).await,
        ["user", "reset", username] => reset(pool, username).await,
        ["ip", "list"] => list_blocked(pool).await,
        ["ip", "block", address] => block_address(pool, address).await,
        ["ip", "unblock", address] => unblock_address(pool, address).await,
        _ => {
            eprintln!("{USAGE}");
            std::process::exit(2);
        }
    }
}

/// Lets a locked account log in again
async fn unlock(pool: &SqlitePool, username: &str) -> Result<()> {

    // Determine if account exists
    let locked: Option<bool> = sqlx::query_scalar("SELECT locked FROM users WHERE username = ?1")
        .bind(username)
        .fetch_optional(pool)
        .await?;

    // Provide relevant messaging if account cannot be modified
    match locked {
        None => {
            eprintln!("no account named \"{username}\"");
            std::process::exit(1);
        }
        Some(false) => {
            eprintln!("account \"{username}\" is not locked");
            std::process::exit(1);
        }
        Some(true) => {}
    }

    // Unlock user account
    sqlx::query("UPDATE users SET locked = 0 WHERE username = ?1")
        .bind(username)
        .execute(pool)
        .await?;
    println!("unlocked user account: \"{username}\"");
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

/// Prints every blocked address
async fn list_blocked(pool: &SqlitePool) -> Result<()> {
    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "
        SELECT address, reason, strftime('%Y-%m-%d %H:%M:%S', created_at / 1000, 'unixepoch', 'localtime')
        FROM blocked_addresses
        ORDER BY created_at
        "
    )
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        println!("no blocked addresses");
        return Ok(())
    }

    // Print list with header
    let left = 40; let middle = 14;
    println!("{:<left$}{:<middle$}{}", "Address", "Reason", "Created At");
    for (address, reason, created_at) in rows {
        println!("{:<left$}{:<middle$}{}", address, reason, created_at);
    }

    Ok(())
}

/// Blocks an address
async fn block_address(pool: &SqlitePool, address: &str) -> Result<()> {

    let Ok(address) = address.parse::<IpAddr>() else {
        eprintln!("\"{address}\" is not an IP address");
        std::process::exit(1);
    };

    let mut conn = pool.acquire().await?;
    if !db::save_blocked_address(&mut conn, address, "manual").await? {
        eprintln!("address \"{address}\" is already blocked");
        std::process::exit(1);
    }

    println!("blocked address: \"{address}\"");
    Ok(())
}

/// Lets a blocked address make requests again
async fn unblock_address(pool: &SqlitePool, address: &str) -> Result<()> {

    // Parse so the address is written the same way it was saved
    let Ok(address) = address.parse::<IpAddr>() else {
        eprintln!("\"{address}\" is not an IP address");
        std::process::exit(1);
    };

    let deleted = sqlx::query("DELETE FROM blocked_addresses WHERE address = ?1")
        .bind(address.to_string())
        .execute(pool)
        .await?
        .rows_affected();

    if deleted == 0 {
        eprintln!("address \"{address}\" is not blocked");
        std::process::exit(1);
    }

    println!("unblocked address: \"{address}\"");
    Ok(())
}
