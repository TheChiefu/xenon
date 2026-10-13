use reqwest::StatusCode;
use serde::Deserialize;

use crate::config;
use crate::services::CLIENT;
use crate::services::xbox::accounts;
use crate::services::xbox::accounts::Account;
use crate::services::xbox::mappings;
use crate::services::xbox::mappings::{Mapping, Mappings};
use crate::services::xbox::session;
use crate::shared::Platform;
use crate::sockets::events::{GameActivity, GamePresence};
use crate::sockets::game_presence;
use crate::state::AppState;

const PRESENCE_URL: &str = "https://userpresence.xboxlive.com/users";

// Data Types //

/// Rich presence a title reports
#[derive(Deserialize)]
struct Activity {
    #[serde(rename = "richPresence")]
    rich_presence: Option<String>
}

/// One title a device reports
#[derive(Deserialize)]
struct Title {
    name: String,

    /// Full for whichever title is in the foreground
    placement: String,

    activity: Option<Activity>
}

/// One device an account appears on
#[derive(Deserialize)]
struct Device {
    #[serde(default)]
    titles: Vec<Title>
}

/// What Xbox reports about an account
#[derive(Deserialize)]
struct Record {
    /// Online, Away or Offline, in Xbox's own words
    state: String,

    #[serde(default)]
    devices: Vec<Device>
}

// Public Methods //

/// Read presence for every linked account and announce it
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
        poll_one(state, &account, xbox, mappings).await;
    }
}

// Helper Methods //

/// Read presence for one account
async fn poll_one(
    state: &AppState,
    account: &Account,
    xbox: &config::Xbox,
    mappings: &Mappings,
) {
    let Some(authorization) = session::authorization(&state.pool, account, xbox).await else {
        return;
    };

    let Some(record) = fetch(&account.xuid, &authorization).await else {
        return;
    };

    let game = reported(record, mappings);
    game_presence::on_report(state, account.user_id, game).await;
}

/// Request one account's presence record
async fn fetch(xuid: &str, authorization: &str) -> Option<Record> {
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

    if response.status() == StatusCode::UNAUTHORIZED {
        tracing::info!("{xuid}: presence header was refused");
        return None;
    }

    if !response.status().is_success() {
        let status = response.status();
        tracing::warn!("{xuid}: presence request answered {status}");
        return None;
    }

    match response.json::<Record>().await {
        Ok(record) => Some(record),
        Err(e) => {
            tracing::warn!("{xuid}: presence record was unreadable: {e}");
            None
        }
    }
}

/// What a presence record shows, with listed names replacing reported ones
fn reported(record: Record, mappings: &Mappings) -> GameActivity {
    let status = match record.state.as_str() {
        "Online" => GamePresence::Online,
        "Away" => GamePresence::Away,
        _ => GamePresence::Offline
    };

    let mut game = GameActivity {
        platform: Platform::Xbox,
        status,
        title: None,
        detail: None
    };

    // Only an online account is in a game
    if status != GamePresence::Online {
        return game;
    }

    // An account appears on several devices at once, and the title in the
    // foreground is the game, the rest being the shell and the mobile app
    for device in record.devices {
        for title in device.titles {
            if title.placement != "Full" {
                continue;
            }

            let mapping = mappings.get(&mappings::lookup_key(&title.name));
            game.detail = detail(title.activity, mapping);
            game.title = Some(name(&title.name, mapping));
            return game;
        }
    }

    game
}

/// Name shown for a title
fn name(reported: &str, mapping: Option<&Mapping>) -> String {
    match mapping {
        Some(mapping) => mapping.name.clone(),
        None => mappings::remove_excessive(reported)
    }
}

/// Rich presence shown for a title, when it reports one
fn detail(activity: Option<Activity>, mapping: Option<&Mapping>) -> Option<String> {
    let rich_presence = activity?.rich_presence?;

    match mapping {
        Some(mapping) => Some(mappings::replace_prefix(&mapping.activity, &rich_presence)),
        None => Some(rich_presence)
    }
}
