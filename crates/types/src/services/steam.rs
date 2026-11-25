//! Steam account linking

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "services/steam.ts"))]
pub struct LinkResponse {
    pub login_url: String
}
