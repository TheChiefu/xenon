//! Registration, login, and registration codes

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

// REQUESTS //

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct RequestRegistrationCreate {
    pub max_uses: Option<i64>,
    pub lifetime: Option<i64>,
    pub length: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct RequestRegistrationRevoke {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct RequestLogin {
    pub username: String,
    pub password: String,
}

// RESPONSES //

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct RespondRegistration {
    pub id: Uuid,
    pub session_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct RespondLogin {
    pub session_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "routes/auth.ts"))]
pub struct RespondRegistrationCode {
    pub code: String,
}
