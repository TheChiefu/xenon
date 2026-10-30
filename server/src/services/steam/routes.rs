use std::collections::{BTreeMap, HashMap};
use std::sync::RwLock;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use url::form_urlencoded::Serializer;
use uuid::Uuid;

use crate::config;
use crate::error::{AppError, Result};
use crate::routes::AuthUser;
use crate::services::CLIENT;
use crate::services::steam;
use crate::services::steam::{accounts, api, REDIRECT_PATH};
use crate::state::AppState;
use crate::utils;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

// Constants //
macro_rules! steam_openid_url {
    ($suffix:literal) => { concat!("https://steamcommunity.com/openid/", $suffix) };
}
const STEAM_ID_URL: &str = steam_openid_url!("id/");
const STEAM_LOGIN_URL: &str = steam_openid_url!("login");

const OPENID_MODE: &str = "openid.mode";
const OPENID_CLAIMED_ID: &str = "openid.claimed_id";
const OPENID_IDENTIFIER_SELECT: &str = "http://specs.openid.net/auth/2.0/identifier_select";

const XENON_USER_ID: &str = "user_id";
const XENON_TOKEN: &str = "token";

/// Page shown after linking, in tab client opened
const LINKED_PAGE: &str = r#"
<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <style>
    p, button {
    font-family: Noto Sans, sans-serif
    }
    html, body {
      height: 100%;
      margin: 0;
      background: rgba(0, 40, 100, 1);
    }
    body {
      display: flex;
      align-items: center;
      justify-content: center;
      font-family: sans-serif;
    }
    .box {
      border: 1px solid rgba(85, 85, 85, 1);
      border-radius: 8px;
      background: rgba(200, 200, 200, 1);
      padding: 24px;
      max-width: 400px;
      text-align: center;
    }
  </style>
</head>
<body>
  <div class="box">
    <p>Account linked</p>
    <p>Click refresh icon on profile to update</p>
    <button onclick="window.close()">Return to  Client</button>
  </div>
</body>
</html>
"#;

// State //

// Accounts related to steam login sessions
static LINK_TOKENS: RwLock<BTreeMap<Uuid, String>> = RwLock::new(BTreeMap::new());

// Data Types //

#[derive(Debug, Deserialize)]
pub struct RedirectQuery {
    user_id: Uuid,
    token: String,
    #[serde(flatten)]
    params: HashMap<String, String>
}

#[derive(Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "services/steam.ts"))]
pub struct LinkResponse {
    pub login_url: String
}

// Public Methods //

pub async fn start(
    AuthUser(user_id, ..): AuthUser,
) -> Json<LinkResponse> {
    let token = utils::generate_code(64);
    let login_url = login_url(user_id, &token);
    utils::write_lock(&LINK_TOKENS).insert(user_id, token);
    Json(LinkResponse { login_url })
}

pub async fn redirect(
    State(pool): State<SqlitePool>,
    Query(query): Query<RedirectQuery>,
) -> Response {
    match link(&pool, query).await {
        Ok(()) => Html(LINKED_PAGE).into_response(),
        Err(e) => {
            tracing::warn!("Steam linking failed: {e}");
            let (status, message) = e.status_and_message();
            (status, format!("linking failed: {message}")).into_response()
        }
    }
}

pub async fn unlink(
    AuthUser(user_id, ..): AuthUser,
    State(state): State<AppState>,
) -> Result<StatusCode> {
    steam::unlink(&state, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Helper Methods //

// Construct Steam login url for user to sign in at
fn login_url(user_id: Uuid, token: &str) -> String {
    let public_url = config::get().public_url();

    let return_query = Serializer::new(String::new())
        .extend_pairs([
            (XENON_USER_ID, user_id.to_string().as_str()),
            (XENON_TOKEN, token)
        ])
        .finish();
    let return_to = format!("{public_url}{REDIRECT_PATH}?{return_query}");

    let login_query = Serializer::new(String::new())
        .extend_pairs([
            ("openid.ns", "http://specs.openid.net/auth/2.0"),
            (OPENID_MODE, "checkid_setup"),
            ("openid.return_to", return_to.as_str()),
            ("openid.realm", public_url),
            ("openid.identity", OPENID_IDENTIFIER_SELECT),
            (OPENID_CLAIMED_ID, OPENID_IDENTIFIER_SELECT),
        ])
        .finish();

    format!("{STEAM_LOGIN_URL}?{login_query}")
}

// Attempt to link account to steam and parse steam ID
async fn link(pool: &SqlitePool, query: RedirectQuery) -> Result<()> {
    let token = utils::write_lock(&LINK_TOKENS).remove(&query.user_id);

    if token.as_deref() != Some(query.token.as_str()) {
        return Err(AppError::Forbidden);
    }

    verify(&query.params).await?;
    let steam_id = parse_steam_id(&query.params)?;
    let mut summaries = api::player_summaries(&[steam_id.as_str()]).await?;

    let Some(summary) = summaries.pop() else {
        return Err(AppError::NotFound);
    };

    accounts::save(pool, query.user_id, &steam_id, &summary.personaname).await?;

    Ok(())
}

// POST to steam to verify that credentials and OpenID parameters are valid
async fn verify(params: &HashMap<String, String>) -> Result<()> {
    let mut params = params.clone();
    params.insert(
        OPENID_MODE.to_string(),
        "check_authentication".to_string()
    );

    let body = CLIENT
        .post(STEAM_LOGIN_URL)
        .form(&params)
        .send()
        .await?
        .text()
        .await?;

    for line in body.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };

        if key == "is_valid" && value.parse::<bool>() == Ok(true) {
            return Ok(());
        }
    }

    Err(AppError::Forbidden)
}

// Given query parameters, extract Steam ID (64)
fn parse_steam_id(params: &HashMap<String, String>) -> Result<String> {
    let Some(claimed_id) = params.get(OPENID_CLAIMED_ID) else {
        return Err(AppError::Validation(format!("Missing {OPENID_CLAIMED_ID}")));
    };

    let Some(steam_id) = claimed_id.strip_prefix(STEAM_ID_URL) else {
        return Err(AppError::Validation(format!("Invalid {OPENID_CLAIMED_ID}: {claimed_id}")));
    };

    if steam_id.parse::<u64>().is_err() {
        return Err(AppError::Validation(format!("Invalid {OPENID_CLAIMED_ID}: {claimed_id}")));
    }

    Ok(steam_id.to_string())
}
