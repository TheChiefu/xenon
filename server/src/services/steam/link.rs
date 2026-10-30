//! Linking a Steam account through Steam's OpenID sign-in

use std::collections::{BTreeMap, HashMap};
use std::sync::RwLock;

use sqlx::SqlitePool;
use url::form_urlencoded::Serializer;
use uuid::Uuid;

use crate::config;
use crate::error::{AppError, Result};
use crate::services::CLIENT;
use crate::services::steam::{accounts, api, REDIRECT_PATH};
use crate::utils;

// CONSTANTS //
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

// STATE //

/// Accounts related to steam login sessions
static LINK_TOKENS: RwLock<BTreeMap<Uuid, String>> = RwLock::new(BTreeMap::new());

// PUBLIC METHODS //

/// Start linking, returning the Steam login url for user to sign in at
pub fn begin(user_id: Uuid) -> String {
    let token = utils::generate_code(64);
    let login_url = login_url(user_id, &token);
    utils::write_lock(&LINK_TOKENS).insert(user_id, token);
    login_url
}

/// Attempt to link account to steam and parse steam ID
pub async fn complete(
    pool: &SqlitePool,
    user_id: Uuid,
    token: &str,
    params: &HashMap<String, String>,
) -> Result<()> {
    let expected = utils::write_lock(&LINK_TOKENS).remove(&user_id);

    if expected.as_deref() != Some(token) {
        return Err(AppError::Forbidden);
    }

    verify(params).await?;
    let steam_id = parse_steam_id(params)?;
    let mut summaries = api::player_summaries(&[steam_id.as_str()]).await?;

    let Some(summary) = summaries.pop() else {
        return Err(AppError::NotFound);
    };

    accounts::save(pool, user_id, &steam_id, &summary.personaname).await?;

    Ok(())
}

// HELPER METHODS //

/// Construct Steam login url for user to sign in at
fn login_url(user_id: Uuid, token: &str) -> String {
    let public_url = config::get().public_url();

    let return_query = Serializer::new(String::new())
        .extend_pairs([
            (XENON_USER_ID, user_id.to_string().as_str()),
            (XENON_TOKEN, token)
        ])
        .finish();
    let return_to = format!("{}{}?{}", public_url, REDIRECT_PATH, return_query);

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

/// POST to steam to verify that credentials and OpenID parameters are valid
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

/// Given query parameters, extract Steam ID (64)
fn parse_steam_id(params: &HashMap<String, String>) -> Result<String> {
    let Some(claimed_id) = params.get(OPENID_CLAIMED_ID) else {
        return Err(AppError::Validation(format!("missing {OPENID_CLAIMED_ID}")));
    };

    let Some(steam_id) = claimed_id.strip_prefix(STEAM_ID_URL) else {
        return Err(AppError::Validation(format!("invalid {OPENID_CLAIMED_ID}: {claimed_id}")));
    };

    if steam_id.parse::<u64>().is_err() {
        return Err(AppError::Validation(format!("invalid {OPENID_CLAIMED_ID}: {claimed_id}")));
    }

    Ok(steam_id.to_string())
}
