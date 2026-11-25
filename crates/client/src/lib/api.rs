//! Requests to a Xenon server

use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use types::auth::Registration;
use types::error::ErrorResponse;
use types::routes::auth::{RequestLogin, RespondLogin, RespondRegistration};
use types::users::UserProfile;

/// Text explaining why a request failed
pub type ErrorMessage = String;

/// Sign in given credentials
/// Returns: Session string or failure string
pub async fn login(
    server: &str,
    username: &str,
    password: &str
) -> Result<String, ErrorMessage> {

    // Construct URL and request body
    let url = format!("{server}/login");
    let request = RequestLogin {
        username: username.to_string(),
        password: password.to_string(),
    };

    let sent = Client::new()
        .post(url)
        .json(&request)
        .send()
        .await;

    let login: RespondLogin = read(sent).await?;
    Ok(login.session_token)
}

/// Create an account
/// Returns: Session string or failure string
pub async fn register(server: &str, registration: &Registration) -> Result<String, ErrorMessage> {
    let url = format!("{server}/users");

    let sent = Client::new()
        .post(url)
        .json(registration)
        .send()
        .await;

    let registered: RespondRegistration = read(sent).await?;
    Ok(registered.session_token)
}

/// Get the signed-in user's profile
pub async fn me(server: &str, session: &str) -> Result<UserProfile, ErrorMessage> {
    let url = format!("{server}/me");

    let sent = Client::new()
        .get(url)
        .bearer_auth(session)
        .send()
        .await;

    read(sent).await
}

/// End a session on the server
pub async fn sign_out(server: &str, session: &str) -> Result<(), ErrorMessage> {
    let url = format!("{server}/me/session");

    let sent = Client::new()
        .delete(url)
        .bearer_auth(session)
        .send()
        .await;

    match sent {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Read a response body as `T`, or the server's error message when the request failed
async fn read<T: DeserializeOwned>(
    sent: reqwest::Result<Response>
) -> Result<T, ErrorMessage> {
    let response = match sent {
        Ok(response) => response,
        Err(e) => return Err(e.to_string()),
    };

    if !response.status().is_success() {
        return match response.json::<ErrorResponse>().await {
            Ok(body) => Err(body.error),
            Err(e) => Err(e.to_string()),
        };
    }

    match response.json::<T>().await {
        Ok(body) => Ok(body),
        Err(e) => Err(e.to_string()),
    }
}
