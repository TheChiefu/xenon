//! Requests to a Xenon server

use reqwest::Client;
use slint::SharedString;
use types::error::ErrorResponse;
use types::routes::auth::{RequestLogin, RespondLogin};

use crate::errors::ErrorMessage;
use crate::session::Session;

/// Signs in, returning the new session or the reason it failed
pub async fn login(
    server: SharedString,
    username: SharedString,
    password: SharedString,
) -> Result<Session, ErrorMessage> {
    let url = format!("{}/login", server.trim_end_matches('/'));
    let request = RequestLogin {
        username: username.to_string(),
        password: password.to_string(),
    };

    let sent = Client::new()
        .post(url)
        .json(&request)
        .send()
        .await;

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

    match response.json::<RespondLogin>().await {
        Ok(body) => Ok(body.session_token),
        Err(e) => Err(e.to_string()),
    }
}
