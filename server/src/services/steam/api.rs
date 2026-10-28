use serde::Deserialize;

use crate::config;
use crate::error::Result;

const PLAYER_SUMMARIES_URL: &str = "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/";

#[derive(Deserialize)]
struct SummariesResponse {
    response: Summaries
}

#[derive(Deserialize)]
struct Summaries {
    players: Vec<PlayerSummary>
}

#[derive(Deserialize)]
pub struct PlayerSummary {
    pub steamid: String,
    pub personaname: String,
    pub personastate: PersonaState,
    pub gameextrainfo: Option<String>
}

#[derive(Deserialize)]
#[serde(from = "u8")]
pub enum PersonaState {
    Offline,
    Online,
    Busy,
    Away,
    Snooze,
    LookingToTrade,
    LookingToPlay
}

impl From<u8> for PersonaState {
    fn from(code: u8) -> PersonaState {
        match code {
            1 => PersonaState::Online,
            2 => PersonaState::Busy,
            3 => PersonaState::Away,
            4 => PersonaState::Snooze,
            5 => PersonaState::LookingToTrade,
            6 => PersonaState::LookingToPlay,
            _ => PersonaState::Offline
        }
    }
}

pub async fn player_summaries(steam_ids: &[&str]) -> Result<Vec<PlayerSummary>> {
    let web_api_key = &config::get().steam().web_api_key;

    let url = reqwest::Url::parse_with_params(PLAYER_SUMMARIES_URL, [
        ("key", web_api_key.as_str()),
        ("steamids", steam_ids.join(",").as_str()),
    ])?;

    let summaries: SummariesResponse = reqwest::Client::new()
        .get(url)
        .send()
        .await?
        .json()
        .await?;

    Ok(summaries.response.players)
}
