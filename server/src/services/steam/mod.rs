// Steam account linking and presence polling
pub mod routes;

use axum::Router;
use axum::routing::get;

use crate::{config, state::AppState};

pub fn router() -> Router<AppState> {
    if config::get().steam.is_none() {
        return Router::new();
    }

    Router::new()
        .route("/steam/redirect", get(routes::redirect))
}