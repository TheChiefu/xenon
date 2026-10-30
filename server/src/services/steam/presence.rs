//! Steam presence polling for linked accounts

use crate::services::steam::accounts;
use crate::services::steam::api::{self, PersonaState, PlayerSummary};
use crate::state::platforms;
use crate::state::AppState;
use crate::types::platforms::{Availability, Metadata, Platform};

const SUMMARIES_PER_REQUEST: usize = 100;

pub async fn poll(state: &AppState) {
    let linked = match accounts::all(&state.pool).await {
        Ok(linked) => linked,
        Err(e) => {
            tracing::error!("could not read linked Steam accounts: {e}");
            return;
        }
    };

    for batch in linked.chunks(SUMMARIES_PER_REQUEST) {
        let mut steam_ids = Vec::new();
        for account in batch {
            steam_ids.push(account.steam_id.as_str());
        }

        let summaries = match api::player_summaries(&steam_ids).await {
            Ok(summaries) => summaries,
            Err(e) => {
                tracing::warn!("presence request to Steam failed: {e}");
                continue;
            }
        };

        for summary in summaries {
            for account in batch {
                if account.steam_id == summary.steamid {
                    let metadata = reported(&summary);
                    platforms::report_metadata(state, account.user_id, Platform::Steam, metadata).await;
                }
            }
        }
    }
}

fn reported(summary: &PlayerSummary) -> Metadata {
    let availability = match summary.personastate {
        PersonaState::Offline => Availability::Offline,
        PersonaState::Online => Availability::Online,
        PersonaState::Busy => Availability::Online,
        PersonaState::Away => Availability::Away,
        PersonaState::Snooze => Availability::Away,
        PersonaState::LookingToTrade => Availability::Online,
        PersonaState::LookingToPlay => Availability::Online
    };

    Metadata {
        availability,
        game: summary.gameextrainfo.clone(),
        activity: None
    }
}
