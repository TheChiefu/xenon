//! Types used for authentication

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

#[derive(Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/auth.ts"))]
pub struct Registration {
    pub registration_code: Option<String>,
    pub username: String,
    pub password: String,
    pub display_name: String,
}

#[derive(sqlx::FromRow, Serialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "types/auth.ts"))]
pub struct RegistrationCode {
    pub code: String,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub max_uses: Option<i64>,
    pub uses: i64,
}
