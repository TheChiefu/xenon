//! Push subscriptions

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

/// A browser's push subscription
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "services/push.ts"))]
pub struct Subscription {
    /// Where to send to
    pub endpoint: String,

    /// Browser's public key
    pub p256dh: Vec<u8>,

    /// Browser's secret
    pub auth: Vec<u8>
}

/// The key browsers subscribe against
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "services/push.ts"))]
pub struct VapidKeyResponse {
    pub public_key: Vec<u8>
}

/// Names a subscription to remove
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "services/push.ts"))]
pub struct UnsubscribeRequest {
    pub endpoint: String
}
