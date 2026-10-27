use std::{collections::{BTreeMap, HashMap}, sync::RwLock};

use axum::{extract::{Query, State}, http::StatusCode, response::{Html, IntoResponse, Response}, Json};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool};
use uuid::Uuid;

use crate::{config, error::{AppError, Result}, routes::AuthUser};
use crate::services::steam::{accounts, api, REDIRECT_PATH};
use crate::utils::{generate_code, write_lock};

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

// Constants //

const STEAM_LOGIN_URL: &str = "https://steamcommunity.com/openid/login";
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
) -> Result<Json<LinkResponse>> {
    let token = generate_code(64);
    let login_url = login_url(user_id, &token)?;
    write_lock(&LINK_TOKENS).insert(user_id, token);
    Ok(Json(LinkResponse { login_url }))
}

pub async fn redirect(
    State(pool): State<SqlitePool>,
    Query(query): Query<RedirectQuery>,
) -> Response {
    match link(&pool, query).await {
        Ok(()) => Html(LINKED_PAGE).into_response(),
        Err(e) => {
            tracing::warn!("Steam linking failed: {e}");
            format!("Steam linking failed: {e}").into_response()
        }
    }
}

pub async fn unlink(
    AuthUser(user_id, ..): AuthUser,
    State(pool): State<SqlitePool>,
) -> Result<StatusCode> {
    remove(&pool, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove(pool: &SqlitePool, user_id: Uuid) -> Result<()> {
    write_lock(&LINK_TOKENS).remove(&user_id);
    accounts::delete(pool, user_id).await
}

// Helper Methods //

// Construct Steam login url for user to sign in at
fn login_url(user_id: Uuid, token: &str) -> Result<String> {
    let public_url = config::get().public_url();

    let return_to = reqwest::Url::parse_with_params(
        &format!("{public_url}{REDIRECT_PATH}"),
        [(XENON_USER_ID, user_id.to_string().as_str()), (XENON_TOKEN, token)]
    )?;

    let login_url = reqwest::Url::parse_with_params(STEAM_LOGIN_URL, [
        ("openid.ns", "http://specs.openid.net/auth/2.0"),
        (OPENID_MODE, "checkid_setup"),
        ("openid.return_to", return_to.as_str()),
        ("openid.realm", public_url),
        ("openid.identity", OPENID_IDENTIFIER_SELECT),
        (OPENID_CLAIMED_ID, OPENID_IDENTIFIER_SELECT),
    ])?
    .to_string();

    Ok(login_url)
}

// Attempt to link account to steam and parse steam ID
async fn link(pool: &SqlitePool, query: RedirectQuery) -> Result<()> {
    let token = write_lock(&LINK_TOKENS).remove(&query.user_id);

    if token.as_deref() != Some(query.token.as_str()) {
        return Err(AppError::Forbidden);
    }

    verify(&query.params).await?;
    let steam_id = parse_steam_id(&query.params)?;
    let mut summaries = api::player_summaries(&[steam_id]).await?;

    let Some(summary) = summaries.pop() else {
        return Err(AppError::NotFound);
    };

    accounts::save(pool, query.user_id, steam_id, &summary.personaname).await?;

    Ok(())
}

// POST to steam to verify that credentials and OpenID parameters are valid
async fn verify(params: &HashMap<String, String>) -> Result<()> {
    let mut params = params.clone();
    params.insert(
        OPENID_MODE.to_string(),
        "check_authentication".to_string()
    );

    let body = reqwest::Client::new()
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
fn parse_steam_id(params: &HashMap<String, String>) -> Result<u64> {
    let claimed_id = params
        .get(OPENID_CLAIMED_ID)
        .ok_or_else(|| AppError::Validation(
            format!("Missing {OPENID_CLAIMED_ID}")
        ))?;

    claimed_id
        .rsplit('/')
        .next()
        .and_then(|id| id.parse::<u64>().ok())
        .ok_or_else(|| AppError::Validation(
            format!("Invalid {OPENID_CLAIMED_ID}: {claimed_id}")
        ))
}
