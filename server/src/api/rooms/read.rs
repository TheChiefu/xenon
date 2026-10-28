//! How far each member has read in a room

use uuid::Uuid;

use crate::config;
use crate::error::{AppError, Result};

/// A user's stored read position in one room
#[derive(sqlx::FromRow)]
struct Position {
    room_id: Uuid,
    last_read_seq: i64,
}

/// A member's read position in one room and how many messages are past it
pub struct Entry {
    pub room_id: Uuid,
    pub last_read_seq: i64,
    pub unread: i64,
}

/// Lists the user's read position in each room they have one for
pub async fn list(
    pool: &sqlx::SqlitePool,
    user_id: Uuid,
) -> Result<Vec<Entry>> {

    let mut conn = pool.acquire().await?;

    let positions: Vec<Position> = sqlx::query_as(
        "
        SELECT room_id, last_read_seq
        FROM read_state
        WHERE user_id = ?1
        "
    )
    .bind(user_id)
    .fetch_all(&mut *conn)
    .await?;

    let mut entries = Vec::with_capacity(positions.len());
    for position in positions {
        let unread = count_unread(&mut conn, &position, user_id).await?;

        entries.push(Entry {
            room_id: position.room_id,
            last_read_seq: position.last_read_seq,
            unread
        });
    }

    Ok(entries)
}

/// Starts a new member's read position at the room's newest message
pub async fn start(
    conn: &mut sqlx::SqliteConnection,
    room_id: Uuid,
    user_id: Uuid,
) -> Result<()> {

    // Keeps the existing position when the user is already a member
    sqlx::query(
        "
        INSERT INTO read_state (user_id, room_id, last_read_seq)
        SELECT ?1, ?2, COALESCE(MAX(seq), 0)
        FROM messages
        WHERE room_id = ?2
        ON CONFLICT DO NOTHING
        "
    )
    .bind(user_id)
    .bind(room_id)
    .execute(&mut *conn)
    .await?;

    Ok(())
}

/// Stores a member's read position in a room
pub async fn mark(
    pool: &sqlx::SqlitePool,
    room_id: Uuid,
    user_id: Uuid,
    last_read_seq: i64,
) -> Result<()> {

    let mut conn = pool.acquire().await?;

    // Inserts only for a member of the room
    // Keeps the newer position when another device sends an older one
    let written = sqlx::query(
        "
        INSERT INTO read_state (user_id, room_id, last_read_seq)
        SELECT user_id, room_id, ?3
        FROM room_access
        WHERE user_id = ?1 AND room_id = ?2
        ON CONFLICT (user_id, room_id)
        DO UPDATE SET last_read_seq = MAX(last_read_seq, excluded.last_read_seq)
        "
    )
    .bind(user_id)
    .bind(room_id)
    .bind(last_read_seq)
    .execute(&mut *conn)
    .await?
    .rows_affected();

    if written == 0 {
        return Err(AppError::Forbidden);
    }

    Ok(())
}

/// Counts messages past a read position that are not the user's own, up to max
async fn count_unread(
    conn: &mut sqlx::SqliteConnection,
    position: &Position,
    user_id: Uuid,
) -> Result<i64> {

    let unread: i64 = sqlx::query_scalar(
        "
        SELECT COUNT(*) FROM (
            SELECT 1
            FROM messages
            WHERE room_id = ?1
                AND seq > ?2
                AND author_id IS NOT ?3
            LIMIT ?4
        )
        "
    )
    .bind(position.room_id)
    .bind(position.last_read_seq)
    .bind(user_id)
    .bind(config::get().responses.unread_count_max)
    .fetch_one(&mut *conn)
    .await?;

    Ok(unread)
}
