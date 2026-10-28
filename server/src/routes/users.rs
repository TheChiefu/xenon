//! HTTP handlers for users.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

use crate::api::users::UserSummary;
use crate::error::{AppError, Result};
use crate::routes::AuthUser;
use crate::state::{self, AppState};
use crate::types::users::{GlobalRole, UserProfile};
use crate::api;
use crate::config;

// REQUESTS //

#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/users.ts"))]
pub struct RequestRoleSet {
    pub role: GlobalRole,
}

#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/users.ts", optional_fields))]
pub struct RequestUsers {
    #[serde(default)]
    pub ids: Option<Vec<Uuid>>,

    /// Start of a username to match
    #[serde(default)]
    pub username: Option<String>,

    #[serde(default)]
    pub after: Option<Uuid>,

    #[serde(default)]
    pub limit: Option<i64>,
}

// RESPONSES //

#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/users.ts"))]
pub struct RespondSummary {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_file_id: Option<Uuid>,
    pub banner_file_id: Option<Uuid>,
    pub global_role: GlobalRole,
}

impl From<UserSummary> for RespondSummary {
    fn from(user: UserSummary) -> Self {
        Self {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
            avatar_file_id: user.avatar_file_id,
            banner_file_id: user.banner_file_id,
            global_role: user.global_role,
        }
    }
}


// Routing Methods //

/// Get summaries of multiple users
pub async fn query(
    AuthUser(..): AuthUser,
    State(pool): State<SqlitePool>,
    Json(body): Json<RequestUsers>,
) -> Result<Json<Vec<RespondSummary>>> {

    let users = match body.ids {

        // Look up users by id
        Some(ids) => {
            // Reject ids mixed with the search fields
            if body.username.is_some() || body.after.is_some() || body.limit.is_some() {
                return Err(AppError::Validation(
                    "ids cannot be combined with username, after, or limit".to_string()
                ));
            }

            // Reject more ids than the lookup limit
            let max = config::get().responses.users_per_lookup;
            if ids.len() > max {
                return Err(AppError::Validation(format!("at most {max} ids per lookup")));
            }

            api::users::by_ids(&pool, &ids).await?
        }

        // Search users by username prefix, one page at a time
        None => {
            let max = config::get().responses.users_per_page;
            let limit = body.limit.unwrap_or(max).clamp(1, max);

            api::users::by_username_prefix(&pool, body.username, body.after, limit).await?
        }
    };

    Ok(Json(users.into_iter().map(RespondSummary::from).collect()))
}

/// Get a user's public profile
pub async fn get(
    AuthUser(..): AuthUser,
    State(pool): State<SqlitePool>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserProfile>> {
    let profile = api::users::profile(&pool, user_id).await?;
    Ok(Json(profile))
}

/// Promote or demotes a user's global role
pub async fn set_role(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Path(target_id): Path<Uuid>,
    Json(body): Json<RequestRoleSet>,
) -> Result<StatusCode> {
    if !api::users::can_moderate(&pool, caller_id, target_id).await? {
        return Err(AppError::Forbidden);
    }

    api::users::set_global_role(&pool, target_id, body.role).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Ban a user
pub async fn ban(
    AuthUser(caller_id, ..): AuthUser,
    State(app_state): State<AppState>,
    Path(target_id): Path<Uuid>,
) -> Result<StatusCode> {
    if !api::users::can_moderate(&app_state.pool, caller_id, target_id).await? {
        return Err(AppError::Forbidden);
    }

    api::users::ban(&app_state, target_id).await?;

    state::disconnect(&app_state, target_id).await;

    Ok(StatusCode::NO_CONTENT)
}

/// Unban a user
pub async fn unban(
    AuthUser(caller_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
    Path(target_id): Path<Uuid>,
) -> Result<StatusCode> {
    if !api::users::can_moderate(&pool, caller_id, target_id).await? {
        return Err(AppError::Forbidden);
    }

    api::users::unban(&pool, target_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
