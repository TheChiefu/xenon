// SQL DB manager for Xbox accounts
// Sets/Gets accounts derived from Microsoft and Xbox service tokens

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::Result;

/// A linked Xbox account
/// - Xbox account ID (XUID)
/// - refresh token (replaced with one from each token response)
#[derive(sqlx::FromRow)]
pub struct Account {
    pub user_id: Uuid,
    pub xuid: String,
    pub refresh_token: String
}

/// Reads one user's linked gamertag
pub async fn gamertag(pool: &SqlitePool, user_id: Uuid) -> Result<Option<String>> {
    let mut conn = pool.acquire().await?;

    let gamertag = sqlx::query_scalar(
        "
        SELECT gamertag
        FROM xbox_accounts
        WHERE user_id = ?1
        "
    )
    .bind(user_id)
    .fetch_optional(&mut *conn)
    .await?;

    Ok(gamertag)
}

/// Reads every linked account
pub async fn all(pool: &SqlitePool) -> Result<Vec<Account>> {
    let mut conn = pool.acquire().await?;

    let rows = sqlx::query_as(
        "
        SELECT user_id, xuid, refresh_token
        FROM xbox_accounts
        "
    )
    .fetch_all(&mut *conn)
    .await?;

    Ok(rows)
}

/// Stores a linked account (replaces exising one)
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
        INSERT INTO xbox_accounts (user_id, xuid, gamertag, refresh_token)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(user_id) DO UPDATE SET
            xuid = ?2,
            gamertag = ?3,
            refresh_token = ?4
        "
    )
    .bind(user_id)
    .bind(xuid)
    .bind(gamertag)
    .bind(refresh_token)
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
        SET refresh_token = ?2
        WHERE user_id = ?1
        "
    )
    .bind(user_id)
    .bind(refresh_token)
    .execute(&mut *conn)
    .await?;

    Ok(())
}

/// Removes a user's linked account from Database
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
