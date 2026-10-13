use serde::Deserialize;

use crate::services::CLIENT;
use crate::services::xbox::Failure;

const USER_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";

// Data Types //


/// Xbox's own account id, its gamertag, and the hash sent beside the token
#[derive(Deserialize)]
pub struct Claims {
    pub xid: String,
    pub gtg: String,
    pub uhs: String
}

/// One set of claims per signed-in account
#[derive(Deserialize)]
pub struct DisplayClaims {
    pub xui: Vec<Claims>
}

#[derive(Deserialize)]
struct UserAuthBody {
    #[serde(rename = "Token")]
    user_token: String
}

/// What Xbox answers the XSTS call with
#[derive(Deserialize)]
pub struct XstsBody {
    #[serde(rename = "Token")]
    pub token: String,

    #[serde(rename = "NotAfter")]
    pub not_after: String,

    #[serde(rename = "DisplayClaims")]
    pub display_claims: DisplayClaims
}

#[derive(Deserialize)]
struct ErrorBody {
    #[serde(rename = "XErr")]
    xerr: u64
}

// Public Methods //

/// Send an access token to Xbox for a user token, then that for an XSTS token
pub async fn session(access_token: &str) -> Result<XstsBody, Failure> {
    let user_token = fetch_user_token(access_token).await?;

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

    // A rejection names which account state stopped it
    if !response.status().is_success() {
        return Err(failure(response).await);
    }

    match response.json::<XstsBody>().await {
        Ok(parsed) => Ok(parsed),
        Err(e) => Err(Failure::Retry(format!("XSTS response was unreadable: {e}")))
    }
}

// Helper Methods //

/// Trades a Microsoft access token for the user token the XSTS call takes
async fn fetch_user_token(access_token: &str) -> Result<String, Failure> {
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

/// Reads whether a rejected XSTS call ends the account or is worth retrying
async fn failure(response: reqwest::Response) -> Failure {
    let status = response.status();

    let body = match response.json::<ErrorBody>().await {
        Ok(body) => body,
        Err(_) => return Failure::Retry(format!("XSTS request answered {status}"))
    };

    Failure::Dead(xerr_message(body.xerr))
}

/// What an XErr code means to the account holder
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
