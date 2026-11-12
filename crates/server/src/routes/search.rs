//! HTTP handlers for search

use axum::extract::State;
use axum::Json;
use sqlx::SqlitePool;
use types::routes::search::{RespondSearch, RespondSearchResult};
use types::messages::SearchRequest;

use crate::api;
use crate::api::messages::search::Found;
use crate::error::Result;
use crate::routes::AuthUser;

// RESPONSES //

impl From<Found> for RespondSearchResult {
    fn from(found: Found) -> Self {
        Self {
            seq: found.message.seq,
            id: found.message.id,
            room_id: found.message.room_id,
            author_id: found.message.author_id,
            body: found.message.body,
            created_at: found.message.created_at,
            edited_at: found.message.edited_at,
            files: found.files,
        }
    }
}

// ROUTING METHODS //

/// Search for messages in every room the caller is a member of
pub async fn messages(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Json(body): Json<SearchRequest>,
) -> Result<Json<RespondSearch>> {

    let page = api::messages::search::search(&pool, user_id, &body).await?;

    let mut results = Vec::with_capacity(page.found.len());
    for entry in page.found {
        results.push(RespondSearchResult::from(entry));
    }

    Ok(Json(RespondSearch { results, total: page.total }))
}
