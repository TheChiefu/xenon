use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::Result;

#[derive(sqlx::FromRow)]
pub struct Account {
    pub user_id: Uuid,
    pub steam_id: i64,
    pub persona_name: String
}

pub async fn get(pool: &SqlitePool, user_id: Uuid) -> Result<Option<Account>> {
    let row = sqlx::query_as(
        "
        SELECT user_id, steam_id, persona_name
        FROM steam_accounts
        WHERE user_id = ?1
        "
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn delete(pool: &SqlitePool, user_id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM steam_accounts WHERE user_id = ?1")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn save(
    pool: &SqlitePool,
    user_id: Uuid,
    steam_id: u64,
    persona_name: &str,
) -> Result<()> {
    sqlx::query(
        "
        INSERT INTO steam_accounts (user_id, steam_id, persona_name)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(user_id) DO UPDATE SET
            steam_id = ?2,
            persona_name = ?3
        "
    )
    .bind(user_id)
    .bind(steam_id as i64)
    .bind(persona_name)
    .execute(pool)
    .await?;

    Ok(())
}
