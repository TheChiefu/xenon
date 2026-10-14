// Xbox Live friends list for linked accounts

use reqwest::StatusCode;
use serde::Deserialize;

use crate::services::CLIENT;
use crate::services::xbox::accounts::Account;
use crate::services::xbox::auth;
use crate::services::xbox::auth::Failure;
use crate::services::xbox::presence::PresenceState;

const FRIENDS_URL: &str = "https://peoplehub.xboxlive.com/users/me/people/friends/decoration/presenceDetail,multiplayerSummary";

// Data Types //

/// Details about party joinability
/// - none - Joinable by anyone
/// - followed - Joinable
/// - local - Invite Only
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum JoinRestriction {
    None,
    Local,
    Followed,

    #[serde(other)]
    Unknown
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PartyDetails {
    join_restriction: JoinRestriction
}

/// Fields read from multiplayer summary
/// - In party: Number of people in the party
/// - Party details: One entry per party, empty when not in a party
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MultiplayerSummary {
    pub in_party: u32,

    #[serde(default)]
    party_details: Vec<PartyDetails>
}

impl MultiplayerSummary {
    pub fn invite_only(&self) -> bool {
        matches!(
            self.party_details.first(),
            Some(party) if matches!(party.join_restriction, JoinRestriction::Local)
        )
    }
}

/// Fields read from person
/// - Gamertag: Name shown for the account
/// - Display pic raw: Gamerpic URL
/// - Presence state: Overall Xbox status
/// - Presence text: Xbox status line
/// - Multiplayer summary: Party status
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    pub gamertag: String,
    pub display_pic_raw: Option<String>,
    pub presence_state: PresenceState,
    pub presence_text: String,

    #[serde(default)]
    pub multiplayer_summary: MultiplayerSummary
}

/// Fields read from people list
#[derive(Deserialize)]
struct People {
    people: Vec<Person>
}

// Public Methods //

/// Requests friends list for one linked account
pub async fn fetch(account: &Account, authorization: &str) -> Result<Vec<Person>, Failure> {
    let response = CLIENT
        .get(FRIENDS_URL)
        .header("Authorization", authorization)
        .header("x-xbl-contract-version", "7")
        .header("Accept", "application/json")
        .header("Accept-Language", "en-US")
        .send()
        .await
        .map_err(|e| Failure::Retry(format!("friends request failed: {e}")))?;

    // If Authorization header is refused, remove it from cache
    if response.status() == StatusCode::UNAUTHORIZED {
        auth::remove_header(account.user_id);
        return Err(Failure::Retry("friends header was refused".into()));
    }

    if !response.status().is_success() {
        let status = response.status();
        return Err(Failure::Retry(format!("friends request answered {status}")));
    }

    match response.json::<People>().await {
        Ok(people) => Ok(people.people),
        Err(e) => Err(Failure::Retry(format!("friends list was unreadable: {e}")))
    }
}
