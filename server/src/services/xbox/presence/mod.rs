// Xbox Live presence polling for linked accounts
// https://learn.microsoft.com/en-us/gaming/gdk/docs/reference/live/rest/json/json-presencerecord
// https://learn.microsoft.com/en-us/gaming/gdk/docs/reference/live/rest/json/json-devicerecord
// https://learn.microsoft.com/en-us/gaming/gdk/docs/reference/live/rest/json/json-titlerecord
// https://learn.microsoft.com/en-us/gaming/gdk/docs/reference/live/rest/json/json-activityrecord

pub mod mappings;

use reqwest::StatusCode;
use serde::Deserialize;

use crate::config;
use crate::services::CLIENT;
use crate::services::xbox::accounts;
use crate::services::xbox::accounts::Account;
use crate::services::xbox::auth;
use crate::services::xbox::presence::mappings::{Mapping, Mappings};
use crate::shared::Platform;
use crate::sockets::events::platforms::{Presence, Status};
use crate::sockets::chat::platforms;
use crate::state::AppState;

const PRESENCE_URL: &str = "https://userpresence.xboxlive.com/users";

// Data Types //

/// Fields read from title activity
#[derive(Deserialize)]
struct Activity {
    #[serde(rename = "richPresence")]
    rich_presence: Option<String>
}

/// Where a title is shown on screen
#[derive(Deserialize, PartialEq)]
enum Placement {
    Full,
    Fill,
    Snapped,
    Background,

    #[serde(other)]
    Unknown
}

/// Fields read from device title
#[derive(Deserialize)]
struct Title {
    name: String,
    placement: Placement,
    activity: Option<Activity>
}

/// Fields read from presence device
#[derive(Deserialize)]
struct Device {
    #[serde(default)]
    titles: Vec<Title>
}

/// User's activity on Xbox Live
#[derive(Deserialize, PartialEq)]
pub enum PresenceState {
    Online,
    Away,
    Offline,

    #[serde(other)]
    Unknown
}

/// Fields read from presence record
/// - State: Overall Xbox status
/// - Devices: What a user is on and their activity
/// - Cloaked: Whether user appears offline to other Xbox users
#[derive(Deserialize)]
struct Record {
    state: PresenceState,

    #[serde(default)]
    devices: Vec<Device>,

    #[serde(default)]
    cloaked: bool
}

// Public Methods //

/// Polls and reports presence for every linked account
pub async fn poll(
    state: &AppState,
    xbox: &config::Xbox,
    mappings: &Mappings,
) {
    let linked = match accounts::all(&state.pool).await {
        Ok(linked) => linked,
        Err(e) => {
            tracing::error!("could not read linked Xbox accounts: {e}");
            return;
        }
    };

    for account in linked {
        // Skips account if no Authorization header is available
        let Some(authorization) = auth::get_header(&state.pool, &account, xbox).await else {
            continue;
        };

        // Skips account if presence request fails
        let Some(record) = fetch(&account, &authorization).await else {
            continue;
        };

        let presence = reported(record, account.honor_cloaked, mappings);
        platforms::report_presence(state, account.user_id, Platform::Xbox, presence).await;
    }
}

// Helper Methods //

/// Requests presence record for one linked account
async fn fetch(account: &Account, authorization: &str) -> Option<Record> {
    let xuid = &account.xuid;
    let url = format!("{PRESENCE_URL}/xuid({xuid})?level=all");

    let sent = CLIENT
        .get(url)
        .header("Authorization", authorization)
        .header("x-xbl-contract-version", "3")
        .header("Accept", "application/json")
        .header("Accept-Language", "en-US")
        .send()
        .await;

    let response = match sent {
        Ok(response) => response,
        Err(e) => {
            tracing::warn!("{xuid}: presence request failed: {e}");
            return None;
        }
    };

    // If Authorization header is refused, remove it from cache
    if response.status() == StatusCode::UNAUTHORIZED {
        tracing::info!("{xuid}: presence header was refused");
        auth::remove_header(account.user_id);
        return None;
    }

    if !response.status().is_success() {
        let status = response.status();
        tracing::warn!("{xuid}: presence request answered {status}");
        return None;
    }

    // Reads presence record as text
    let text = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            tracing::warn!("{xuid}: presence record was unreadable: {e}");
            return None;
        }
    };

    // DEBUG: Log presence record
    tracing::debug!("{xuid}: presence record: {text}");

    match serde_json::from_str::<Record>(&text) {
        Ok(record) => Some(record),
        Err(e) => {
            tracing::warn!("{xuid}: presence record was unreadable: {e}");
            None
        }
    }
}

/// Converts presence record to platform presence, applying game mappings
fn reported(record: Record, honor_cloaked: bool, mappings: &Mappings) -> Presence {

    // Cloaked users appear offline, hide everything else
    if record.cloaked && honor_cloaked {
        return Presence { status: Status::Offline, game: None, activity: None };
    }

    let status = match record.state {
        PresenceState::Online => Status::Online,
        PresenceState::Away => Status::Away,
        PresenceState::Offline | PresenceState::Unknown => Status::Offline
    };

    let mut presence = Presence {
        status,
        game: None,
        activity: None,
    };

    // Finds game being played, skipping unnamed titles
    for device in record.devices {
        for title in device.titles {
            if title.placement != Placement::Full || title.name.is_empty() {
                continue;
            }

            let mapping = mappings.get(&mappings::lookup_key(&title.name));
            presence.activity = detail(title.activity, mapping);

            // Uses mapped title name, or reported name with symbols removed
            presence.game = Some(match mapping {
                Some(mapping) => mapping.name.clone(),
                None => mappings::remove_excessive(&title.name)
            });

            return presence;
        }
    }

    presence
}

/// Returns rich presence with mapped prefix, if title reports one
fn detail(activity: Option<Activity>, mapping: Option<&Mapping>) -> Option<String> {
    let rich_presence = activity?.rich_presence?;

    match mapping {
        Some(mapping) => Some(mappings::replace_prefix(&mapping.activity, &rich_presence)),
        None => Some(rich_presence)
    }
}
