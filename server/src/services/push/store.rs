use serde::Deserialize;
use sqlx::SqlitePool;
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::error::Result;
use crate::utils;

/// A browser's push subscription
#[derive(Deserialize, sqlx::FromRow)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "services/push.ts"))]
pub struct Subscription {
    /// Where to send to
    pub endpoint: String,

    /// Browser's public key
    pub p256dh: Vec<u8>,

    /// Browser's secret
    pub auth: Vec<u8>
}

/// The VAPID keypair
#[derive(sqlx::FromRow)]
pub struct Keypair {
    /// Given to browsers
    pub public_key: Vec<u8>,

    /// Signs each message
    pub private_key: Vec<u8>
}

/// Reads the keypair
pub async fn keypair(pool: &SqlitePool) -> Result<Option<Keypair>> {
    let mut conn = pool.acquire().await?;

    let row = sqlx::query_as(
        "
        SELECT public_key, private_key
        FROM push_keys
        WHERE id = 1
        "
    )
    .fetch_optional(&mut *conn)
    .await?;

    Ok(row)
}

/// Stores a keypair, keeping any already there
pub async fn set_keypair(
    pool: &SqlitePool,
    public_key: &[u8],
    private_key: &[u8],
) -> Result<()> {
    let mut conn = pool.acquire().await?;

    sqlx::query(
        "
        INSERT INTO push_keys (id, public_key, private_key, created_at)
        VALUES (1, ?1, ?2, ?3)
        ON CONFLICT(id) DO NOTHING
        "
    )
    .bind(public_key)
    .bind(private_key)
    .bind(utils::now_ms())
    .execute(&mut *conn)
    .await?;

    Ok(())
}

/// Stores a subscription
pub async fn save(
    pool: &SqlitePool,
    user_id: Uuid,
    subscription: &Subscription,
) -> Result<()> {
    let mut conn = pool.acquire().await?;

    sqlx::query(
        "
        INSERT INTO push_subscriptions (endpoint, user_id, p256dh, auth)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(endpoint) DO UPDATE SET
            user_id = ?2,
            p256dh = ?3,
            auth = ?4
        "
    )
    .bind(&subscription.endpoint)
    .bind(user_id)
    .bind(&subscription.p256dh)
    .bind(&subscription.auth)
    .execute(&mut *conn)
    .await?;

    Ok(())
}

/// Removes a user's subscription
pub async fn remove(pool: &SqlitePool, user_id: Uuid, endpoint: &str) -> Result<()> {
    let mut conn = pool.acquire().await?;

    sqlx::query(
        "
        DELETE FROM push_subscriptions
        WHERE endpoint = ?1 AND user_id = ?2
        "
    )
    .bind(endpoint)
    .bind(user_id)
    .execute(&mut *conn)
    .await?;

    Ok(())
}

/// Removes every subscription a user holds
pub async fn remove_all(pool: &SqlitePool, user_id: Uuid) -> Result<()> {
    let mut conn = pool.acquire().await?;

    sqlx::query(
        "
        DELETE FROM push_subscriptions
        WHERE user_id = ?1
        "
    )
    .bind(user_id)
    .execute(&mut *conn)
    .await?;

    Ok(())
}

/// Removes a rejected subscription
pub async fn remove_unreachable(pool: &SqlitePool, endpoint: &str) -> Result<()> {
    let mut conn = pool.acquire().await?;

    sqlx::query(
        "
        DELETE FROM push_subscriptions
        WHERE endpoint = ?1
        "
    )
    .bind(endpoint)
    .execute(&mut *conn)
    .await?;

    Ok(())
}

/// Reads the subscriptions of several users
pub async fn for_users(pool: &SqlitePool, user_ids: &[Uuid]) -> Result<Vec<Subscription>> {
    if user_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut conn = pool.acquire().await?;

    let mut builder = sqlx::QueryBuilder::new(
        "
        SELECT endpoint, p256dh, auth
        FROM push_subscriptions
        WHERE user_id IN (
        "
    );

    let mut list = builder.separated(", ");
    for user_id in user_ids {
        list.push_bind(*user_id);
    }
    list.push_unseparated(")");

    let rows = builder.build_query_as().fetch_all(&mut *conn).await?;

    Ok(rows)
}
