use serde::Deserialize;

use crate::config;
use crate::services::CLIENT;
use crate::services::xbox::Failure;

const AUTHORIZE_URL: &str = "https://login.live.com/oauth20_authorize.srf";
const TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";
const SCOPE: &str = "XboxLive.signin XboxLive.offline_access";

// Data Types //

/// What Microsoft answers a token call with
#[derive(Deserialize)]
pub struct Tokens {
    /// Trades for an Xbox user token, lasts about an hour
    pub access_token: String,

    /// Trades for another pair of tokens, lasts months, absent when Microsoft
    /// keeps the one that was sent
    pub refresh_token: Option<String>
}

#[derive(Deserialize)]
struct ErrorBody {
    error: String,
    error_description: Option<String>
}

// Public Methods //

/// Builds the address a user signs in to Microsoft at, carrying the state that
/// identifies the attempt when the browser comes back
pub fn authorize_url(xbox: &config::Xbox, state: &str) -> String {
    let params = [
        ("client_id", xbox.client_id.as_str()),
        ("response_type", "code"),
        ("redirect_uri", xbox.redirect_uri.as_str()),
        ("scope", SCOPE),
        ("state", state)
    ];

    match reqwest::Url::parse_with_params(AUTHORIZE_URL, params) {
        Ok(url) => url.to_string(),
        Err(e) => {
            tracing::error!("could not build the Microsoft sign-in address: {e}");
            String::new()
        }
    }
}

/// Trades the code a sign-in returned for the first pair of tokens
pub async fn exchange_code(code: &str, xbox: &config::Xbox) -> Result<Tokens, Failure> {
    let form = [
        ("client_id", xbox.client_id.as_str()),
        ("client_secret", xbox.client_secret.as_str()),
        ("code", code),
        ("grant_type", "authorization_code"),
        ("redirect_uri", xbox.redirect_uri.as_str()),
        ("scope", SCOPE)
    ];

    post(&form).await
}

/// Trades a stored refresh token for a new pair, invalidating the one sent
pub async fn refresh_tokens(
    refresh_token: &str,
    xbox: &config::Xbox,
) -> Result<Tokens, Failure> {
    let form = [
        ("client_id", xbox.client_id.as_str()),
        ("client_secret", xbox.client_secret.as_str()),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
        ("scope", SCOPE)
    ];

    post(&form).await
}

// Helper Methods //

/// Posts one form to the token endpoint
async fn post(form: &[(&str, &str)]) -> Result<Tokens, Failure> {
    let sent = CLIENT.post(TOKEN_URL).form(form).send().await;

    let response = match sent {
        Ok(response) => response,
        Err(e) => return Err(Failure::Retry(format!("token request failed: {e}")))
    };

    if !response.status().is_success() {
        return Err(failure(response).await);
    }

    match response.json::<Tokens>().await {
        Ok(tokens) => Ok(tokens),
        Err(e) => Err(Failure::Retry(format!("token response was unreadable: {e}")))
    }
}

/// Reads whether a rejected token call ends the account or is worth retrying
async fn failure(response: reqwest::Response) -> Failure {
    let status = response.status();

    let body = match response.json::<ErrorBody>().await {
        Ok(body) => body,
        Err(_) => return Failure::Retry(format!("token request answered {status}"))
    };

    let message = match body.error_description {
        Some(description) => description,
        None => body.error.clone()
    };

    match body.error.as_str() {
        "invalid_grant" => Failure::Dead(message),
        _ => Failure::Retry(message)
    }
}
