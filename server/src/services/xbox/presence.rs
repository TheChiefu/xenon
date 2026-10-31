//! Xbox Live presence polling for linked accounts
//! https://learn.microsoft.com/en-us/gaming/gdk/docs/reference/live/rest/json/json-presencerecord
//! https://learn.microsoft.com/en-us/gaming/gdk/docs/reference/live/rest/json/json-devicerecord
//! https://learn.microsoft.com/en-us/gaming/gdk/docs/reference/live/rest/json/json-titlerecord
//! https://learn.microsoft.com/en-us/gaming/gdk/docs/reference/live/rest/json/json-activityrecord

pub mod mappings;

use reqwest::StatusCode;
use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, AUTHORIZATION};
use serde::Deserialize;

use crate::config;
use crate::services::CLIENT;
use crate::services::xbox::accounts::{self, Account};
use crate::services::xbox;
use crate::services::xbox::auth::{self, Failure};
use crate::services::xbox::presence::mappings::{Mapping, Mappings};
use crate::state::platforms;
use crate::state::AppState;
use crate::types::platforms::{Availability, Metadata, Platform};

const PRESENCE_URL: &str = "https://userpresence.xboxlive.com/users";
const HOME_TITLE_ID: &str = "750323071";

// DATA STRUCTS //

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
    id: String,
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
enum PresenceState {
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

// PUBLIC METHODS //

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
        let authorization = match auth::get_header(&state.pool, &account, xbox).await {
            Ok(authorization) => authorization,
            Err(Failure::Dead(_)) => {
                let _ = xbox::unlink(state, account.user_id).await;
                continue;
            }
            Err(Failure::Retry(_)) => continue,
        };

        // Skips account if presence request fails
        let Some(record) = fetch(&account, &authorization).await else {
            continue;
        };

        let metadata = to_metadata(record, account.honor_cloaked, mappings);
        platforms::report_metadata(state, account.user_id, Platform::Xbox, metadata).await;
    }
}

// HELPER METHODS //

/// Requests presence record for one linked account
async fn fetch(account: &Account, authorization: &str) -> Option<Record> {
    let xuid = &account.xuid;
    let url = format!("{PRESENCE_URL}/xuid({xuid})?level=all");

    let sent = CLIENT
        .get(url)
        .header(AUTHORIZATION, authorization)
        .header(xbox::CONTRACT_VERSION_HEADER, "3")
        .header(ACCEPT, "application/json")
        .header(ACCEPT_LANGUAGE, "en-US")
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
fn to_metadata(record: Record, honor_cloaked: bool, mappings: &Mappings) -> Metadata {

    // Cloaked users appear offline, hide everything else
    if record.cloaked && honor_cloaked {
        return Metadata { availability: Availability::Offline, game: None, activity: None };
    }

    let availability = match record.state {
        PresenceState::Online => Availability::Online,
        PresenceState::Away => Availability::Away,
        PresenceState::Offline | PresenceState::Unknown => Availability::Offline
    };

    let mut metadata = Metadata {
        availability,
        game: None,
        activity: None,
    };

    // Finds the game being played, skipping titles that are not full screen, unnamed, or the home screen
    for device in record.devices {
        for title in device.titles {
            if
                title.placement != Placement::Full ||
                title.name.is_empty() ||
                title.id == HOME_TITLE_ID
            { continue; }

            let mapping = mappings.get(&mappings::lookup_key(&title.name));
            metadata.activity = activity_text(title.activity, mapping);

            // Uses the mapped title name, cleaning unmapped names
            metadata.game = Some(match mapping {
                Some(mapping) => mapping.name.clone(),
                None => mappings::clean_title(&title.name)
            });

            return metadata;
        }
    }

    metadata
}

/// Returns rich presence with mapped prefix, if title reports one
fn activity_text(activity: Option<Activity>, mapping: Option<&Mapping>) -> Option<String> {
    let rich_presence = activity?.rich_presence?;

    match mapping {
        Some(mapping) => Some(mappings::replace_prefix(&mapping.activity, &rich_presence)),
        None => Some(rich_presence)
    }
}
