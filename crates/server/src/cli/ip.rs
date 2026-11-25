//! `ip` commands, managing blocked addresses.

use std::net::IpAddr;

use sqlx::SqlitePool;

use crate::cli;
use crate::db;
use crate::error::{AppError, Result};
use crate::state::AppState;

const COMMANDS: &[&str] = &[
    "ip list",
    "ip block <address>",
    "ip unblock <address>",
];

/// Runs the `ip` command named by the arguments
pub async fn run(state: &AppState, args: &[&str]) -> Result<()> {
    match args {
        ["list"] => list_blocked(&state.pool).await,
        ["block", address] => block_address(state, address).await,
        ["unblock", address] => unblock_address(state, address).await,
        _ => Err(cli::usage(COMMANDS)),
    }
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
    let address_width = 40;
    let reason_width = 14;
    println!("{:<address_width$}{:<reason_width$}{}", "Address", "Reason", "Created At");
    for (address, reason, created_at) in rows {
        println!("{:<address_width$}{:<reason_width$}{}", address, reason, created_at);
    }

    Ok(())
}

/// Blocks an address
async fn block_address(state: &AppState, address: &str) -> Result<()> {
    let address = parse_address(address)?;

    let mut conn = state.pool.acquire().await?;
    if !db::save_blocked_address(&mut conn, address, "manual CLI").await? {
        return Err(AppError::Validation(format!("address \"{address}\" is already blocked")));
    }

    state.address_limits.block(address);
    println!("blocked address: \"{address}\"");
    Ok(())
}

/// Lets a blocked address make requests again
async fn unblock_address(state: &AppState, address: &str) -> Result<()> {
    let address = parse_address(address)?;

    let deleted = sqlx::query("DELETE FROM blocked_addresses WHERE address = ?1")
        .bind(address.to_string())
        .execute(&state.pool)
        .await?
        .rows_affected();

    if deleted == 0 {
        return Err(AppError::Validation(format!("address \"{address}\" is not blocked")));
    }

    state.address_limits.unblock(address);
    println!("unblocked address: \"{address}\"");
    Ok(())
}

// HELPER METHODS //

/// Reads a typed address, written the same way it was saved
fn parse_address(address: &str) -> Result<IpAddr> {
    address.parse().map_err(|_| AppError::Validation(format!("\"{address}\" is not an IP address")))
}
