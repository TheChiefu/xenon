//! Xbox Live half of Xbox linking
//! - Requests a user token, then an XSTS token, starting from a Microsoft access token
//! https://minecraft.wiki/w/Microsoft_authentication
//! https://github.com/OpenXbox/xbox-webapi-python/blob/master/xbox/webapi/authentication/manager.py

use serde::Deserialize;

use crate::services::CLIENT;
use crate::services::xbox::auth::Failure;

const USER_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";

// DATA STRUCTS //

/// Xbox account ID, gamertag, and user hash
#[derive(Deserialize)]
pub struct Claims {
    pub xid: String,
    pub gtg: String,
    pub uhs: String
}

/// One set of display claims per signed-in user
#[derive(Deserialize)]
pub struct DisplayClaims {
    pub xui: Vec<Claims>
}

#[derive(Deserialize)]
struct UserAuthBody {
    #[serde(rename = "Token")]
    user_token: String
}

/// Fields read from XSTS token response

#[derive(Deserialize)]
pub struct XstsBody {
    #[serde(rename = "Token")]
    pub token: String,

    /// An expiry (RFC 3339 time)
    #[serde(rename = "NotAfter")]
    pub not_after: String,

    #[serde(rename = "DisplayClaims")]
    pub display_claims: DisplayClaims
}

impl XstsBody {
    /// Returns display claims if present
    pub fn get_display_claims(&self) -> Option<&Claims> {
        self.display_claims.xui.first()
    }
}

#[derive(Deserialize)]
struct ErrorBody {
    #[serde(rename = "XErr")]
    xerr: u64
}

// PUBLIC METHODS //

/// Requests a user token with a Microsoft access token
pub async fn request_user_token(access_token: &str) -> Result<String, Failure> {
    let body = serde_json::json!({
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT",
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("d={access_token}")
        }
    });

    let sent = CLIENT
        .post(USER_AUTH_URL)
        .header("x-xbl-contract-version", "1")
        .json(&body)
        .send()
        .await;

    let response = match sent {
        Ok(response) => response,
        Err(e) => return Err(Failure::Retry(format!("user token request failed: {e}")))
    };

    if !response.status().is_success() {
        let status = response.status();
        return Err(Failure::Retry(format!("user token request answered {status}")));
    }

    match response.json::<UserAuthBody>().await {
        Ok(parsed) => Ok(parsed.user_token),
        Err(e) => Err(Failure::Retry(format!("user token was unreadable: {e}")))
    }
}

/// Requests an XSTS token with a user token
pub async fn request_xsts_token(user_token: &str) -> Result<XstsBody, Failure> {
    let body = serde_json::json!({
        "RelyingParty": "http://xboxlive.com",
        "TokenType": "JWT",
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [user_token]
        }
    });

    let sent = CLIENT
        .post(XSTS_AUTH_URL)
        .header("x-xbl-contract-version", "1")
        .json(&body)
        .send()
        .await;

    let response = match sent {
        Ok(response) => response,
        Err(e) => return Err(Failure::Retry(format!("XSTS request failed: {e}")))
    };

    // If XSTS request is rejected, read XErr code
    if !response.status().is_success() {
        return Err(failure(response).await);
    }

    match response.json::<XstsBody>().await {
        Ok(parsed) => Ok(parsed),
        Err(e) => Err(Failure::Retry(format!("XSTS response was unreadable: {e}")))
    }
}

// HELPER METHODS //

/// Reads XErr code from rejected XSTS response
async fn failure(response: reqwest::Response) -> Failure {
    let status = response.status();

    // If response has no XErr code, failure is retryable
    let body = match response.json::<ErrorBody>().await {
        Ok(body) => body,
        Err(_) => return Failure::Retry(format!("XSTS request answered {status}"))
    };

    // If response has an XErr code, failure is permanent
    Failure::Dead(xerr_message(body.xerr))
}

/// What an XErr code means
fn xerr_message(xerr: u64) -> String {
    match xerr {
        2148916227 => "account banned".to_string(),
        2148916233 => "no Xbox profile on this Microsoft account".to_string(),
        2148916235 => "Xbox Live unavailable in this region".to_string(),
        2148916236 | 2148916237 => "adult verification required".to_string(),
        2148916238 => "child account not in a family group".to_string(),
        _ => format!("XErr {xerr}")
    }
}
