// Steam account linking and presence polling
pub mod accounts;
pub mod api;
pub mod routes;

use axum::Router;
use axum::routing::{get, post};

use crate::{config, state::AppState};

pub const REDIRECT_PATH: &str = "/steam/redirect";

pub fn router() -> Router<AppState> {
    if config::get().steam.is_none() {
        return Router::new();
    }

    Router::new()
        .route("/me/steam", 
            post(routes::start)
            .delete(routes::unlink)
        )
        .route(REDIRECT_PATH, get(routes::redirect))
}