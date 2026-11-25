//! The session WebSocket handshake

use serde::{Deserialize, Serialize};

use crate::appearance::{Appearance, Device};

/// Query string the handshake carries, read before the upgrade while the
/// request is still an ordinary GET.
#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectQuery {
    #[serde(default)]
    pub device: Option<Device>,
    pub appearance: Appearance,
}
