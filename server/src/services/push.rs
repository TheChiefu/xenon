pub mod routes;
pub mod send;
pub mod store;

use axum::routing::{get, post};
use axum::Router;

use crate::config;
use crate::state::AppState;

/// Routes this service serves
pub fn router() -> Router<AppState> {
    if config::get().push.is_none() {
        return Router::new();
    }

    Router::new()
        .route("/push/vapid", get(routes::vapid_key))
        .route("/me/push",
            post(routes::subscribe)
            .delete(routes::unsubscribe)
        )
}
