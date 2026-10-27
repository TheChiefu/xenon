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
    pub personaname: String
}

pub async fn player_summaries(steam_ids: &[u64]) -> Result<Vec<PlayerSummary>> {
    let web_api_key = &config::get().steam().web_api_key;

    let mut ids = Vec::new();
    for steam_id in steam_ids {
        ids.push(steam_id.to_string());
    }

    let url = reqwest::Url::parse_with_params(PLAYER_SUMMARIES_URL, [
        ("key", web_api_key.as_str()),
        ("steamids", ids.join(",").as_str()),
    ])?;

    let summaries: SummariesResponse = reqwest::Client::new()
        .get(url)
        .send()
        .await?
        .json()
        .await?;

    Ok(summaries.response.players)
}
