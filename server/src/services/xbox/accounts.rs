use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::Result;
use crate::utils::now_ms;

/// A linked Xbox account
#[derive(sqlx::FromRow)]
pub struct Account {
    pub user_id: Uuid,

    /// Xbox's own account id
    pub xuid: String,

    /// Name shown for the account
    pub gamertag: String,

    /// Traded for an access token, rotated on every use
    pub refresh_token: String,

    /// When the refresh token was last rotated
    pub last_refreshed_at: i64
}

/// Reads one user's linked account
pub async fn account(pool: &SqlitePool, user_id: Uuid) -> Result<Option<Account>> {
    let mut conn = pool.acquire().await?;

    let row = sqlx::query_as(
        "
        SELECT user_id, xuid, gamertag, refresh_token, last_refreshed_at
        FROM xbox_accounts
        WHERE user_id = ?1
        "
    )
    .bind(user_id)
    .fetch_optional(&mut *conn)
    .await?;

    Ok(row)
}

/// Reads every linked account
pub async fn all(pool: &SqlitePool) -> Result<Vec<Account>> {
    let mut conn = pool.acquire().await?;

    let rows = sqlx::query_as(
        "
        SELECT user_id, xuid, gamertag, refresh_token, last_refreshed_at
        FROM xbox_accounts
        "
    )
    .fetch_all(&mut *conn)
    .await?;

    Ok(rows)
}

/// Reads the accounts whose refresh token was last rotated before a cutoff
pub async fn stale(pool: &SqlitePool, cutoff: i64) -> Result<Vec<Account>> {
    let mut conn = pool.acquire().await?;

    let rows = sqlx::query_as(
        "
        SELECT user_id, xuid, gamertag, refresh_token, last_refreshed_at
        FROM xbox_accounts
        WHERE last_refreshed_at < ?1
        "
    )
    .bind(cutoff)
    .fetch_all(&mut *conn)
    .await?;

    Ok(rows)
}

/// Stores a linked account, replacing whichever one the user had
pub async fn save(
    pool: &SqlitePool,
    user_id: Uuid,
    xuid: &str,
    gamertag: &str,
    refresh_token: &str,
) -> Result<()> {
    let mut conn = pool.acquire().await?;

    sqlx::query(
        "
        INSERT INTO xbox_accounts (user_id, xuid, gamertag, refresh_token, last_refreshed_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(user_id) DO UPDATE SET
            xuid = ?2,
            gamertag = ?3,
            refresh_token = ?4,
            last_refreshed_at = ?5
        "
    )
    .bind(user_id)
    .bind(xuid)
    .bind(gamertag)
    .bind(refresh_token)
    .bind(now_ms())
    .execute(&mut *conn)
    .await?;

    Ok(())
}

/// Replaces a rotated refresh token
pub async fn set_refresh_token(
    pool: &SqlitePool,
    user_id: Uuid,
    refresh_token: &str,
) -> Result<()> {
    let mut conn = pool.acquire().await?;

    sqlx::query(
        "
        UPDATE xbox_accounts
        SET refresh_token = ?2, last_refreshed_at = ?3
        WHERE user_id = ?1
        "
    )
    .bind(user_id)
    .bind(refresh_token)
    .bind(now_ms())
    .execute(&mut *conn)
    .await?;

    Ok(())
}

/// Removes a user's linked account
pub async fn remove(pool: &SqlitePool, user_id: Uuid) -> Result<()> {
    let mut conn = pool.acquire().await?;

    sqlx::query(
        "
        DELETE FROM xbox_accounts
        WHERE user_id = ?1
        "
    )
    .bind(user_id)
    .execute(&mut *conn)
    .await?;

    Ok(())
}
