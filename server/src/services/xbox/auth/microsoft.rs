//! Microsoft identity platform requests, against v2.0 consumers endpoint
//! - Builds authorization request URL with user ID as state
//! - Redeems an authorization code for tokens, and refreshes tokens with a stored refresh token
//! https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-auth-code-flow

use serde::Deserialize;
use url::form_urlencoded::Serializer;

use crate::config;
use crate::services::CLIENT;
use crate::services::xbox::REDIRECT_PATH;
use crate::services::xbox::auth::Failure;

const AUTHORIZE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const SCOPE: &str = "XboxLive.signin offline_access";

// DATA STRUCTS //

/// Fields read from Microsoft's token response
/// - access token (short lived)
/// - refresh token (replaces old one, needs offline_access scope)

#[derive(Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: Option<String>
}

/// Fields read from Microsoft's error response
#[derive(Deserialize)]
struct ErrorBody {
    error: String,
    error_description: Option<String>
}

// PUBLIC METHODS //

/// Builds authorization request URL with state and code challenge
pub fn authorize_url(xbox: &config::Xbox, state: &str, challenge: &str) -> String {
    let redirect_uri = redirect_uri();
    let query = Serializer::new(String::new())
        .extend_pairs([
            ("client_id", xbox.client_id.as_str()),
            ("response_type", "code"),
            ("redirect_uri", redirect_uri.as_str()),
            ("scope", SCOPE),
            ("state", state),
            ("code_challenge", challenge),
            ("code_challenge_method", "S256")
        ])
        .finish();

    format!("{AUTHORIZE_URL}?{query}")
}

/// Redeems an authorization code for tokens
pub async fn redeem_code(
    code: &str,
    verifier: &str,
    xbox: &config::Xbox,
) -> Result<Tokens, Failure> {
    let redirect_uri = redirect_uri();
    let form = [
        ("client_id", xbox.client_id.as_str()),
        ("client_secret", xbox.client_secret.as_str()),
        ("code", code),
        ("code_verifier", verifier),
        ("grant_type", "authorization_code"),
        ("redirect_uri", redirect_uri.as_str()),
        ("scope", SCOPE)
    ];

    post(&form).await
}

/// Refreshes tokens with a stored refresh token
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

// HELPER METHODS //

fn redirect_uri() -> String {
    format!("{}{}", config::get().public_url(), REDIRECT_PATH)
}

/// Posts form to token endpoint
async fn post(form: &[(&str, &str)]) -> Result<Tokens, Failure> {
    let sent = CLIENT.post(TOKEN_URL).form(form).send().await;

    let response = match sent {
        Ok(response) => response,
        Err(e) => return Err(Failure::Retry(format!("token request failed: {e}")))
    };

    // If token request is rejected, read error code
    if !response.status().is_success() {
        return Err(failure(response).await);
    }

    match response.json::<Tokens>().await {
        Ok(tokens) => Ok(tokens),
        Err(e) => Err(Failure::Retry(format!("token response was unreadable: {e}")))
    }
}

/// Reads error code from rejected token response
async fn failure(response: reqwest::Response) -> Failure {
    let status = response.status();

    // If response has no error body, failure is retryable
    let body = match response.json::<ErrorBody>().await {
        Ok(body) => body,
        Err(_) => return Failure::Retry(format!("token request answered {status}"))
    };

    let message = match body.error_description {
        Some(description) => description,
        None => body.error.clone()
    };

    // If grant is rejected, failure is permanent
    match body.error.as_str() {
        "invalid_grant" => Failure::Dead(message),
        _ => Failure::Retry(message)
    }
}
