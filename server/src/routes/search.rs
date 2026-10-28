//! HTTP handlers for search

use axum::extract::State;
use axum::Json;
use serde::Serialize;
use sqlx::SqlitePool;
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::api;
use crate::api::messages::search::Found;
use crate::error::Result;
use crate::routes::AuthUser;
use crate::types::messages::SearchRequest;

// Data Structs //

#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/search.ts"))]
pub struct SearchResult {
    pub seq: i64,
    pub id: Uuid,
    pub room_id: Uuid,
    pub author_id: Option<Uuid>,
    pub body: Option<String>,
    pub created_at: i64,
    pub edited_at: Option<i64>,
    pub files: Vec<String>, // Attached file names matching the file filter
}

impl From<Found> for SearchResult {
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

// RESPONSES //

#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/search.ts"))]
pub struct RespondSearch {
    pub results: Vec<SearchResult>,
    pub total: i64,
}

// Routing Methods //

/// Search for messages in every room the caller is a member of
pub async fn messages(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Json(body): Json<SearchRequest>,
) -> Result<Json<RespondSearch>> {

    let page = api::messages::search::search(&pool, user_id, &body).await?;

    let mut results = Vec::with_capacity(page.found.len());
    for entry in page.found {
        results.push(SearchResult::from(entry));
    }

    Ok(Json(RespondSearch { results, total: page.total }))
}
