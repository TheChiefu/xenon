//! The `[responses]` section of the config file.

use serde::{Deserialize, Serialize};

/// Most items returned in one response
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Responses {
    pub messages_per_page: i64,
    pub search_results_per_page: i64,
    pub rooms_per_page: i64,
    pub users_per_page: i64,

    /// Most users looked up by id in one request
    pub users_per_lookup: usize,

    /// Unread messages counted per room before counting stops
    pub unread_count_max: i64
}

impl Responses {

    /// Errors on the first invalid value in the section
    pub(super) fn validate(&self) -> Result<(), String> {

        if self.messages_per_page < 1 {
            return Err("responses.messages_per_page must be at least 1".to_string());
        }

        if self.search_results_per_page < 1 {
            return Err("responses.search_results_per_page must be at least 1".to_string());
        }

        if self.rooms_per_page < 1 {
            return Err("responses.rooms_per_page must be at least 1".to_string());
        }

        if self.users_per_page < 1 {
            return Err("responses.users_per_page must be at least 1".to_string());
        }

        if self.users_per_lookup < 1 {
            return Err("responses.users_per_lookup must be at least 1".to_string());
        }

        if self.unread_count_max < 0 {
            return Err("responses.unread_count_max must be at least 0".to_string());
        }

        Ok(())
    }
}

impl Default for Responses {
    fn default() -> Self {
        Responses {
            messages_per_page: 200,
            search_results_per_page: 50,
            rooms_per_page: 200,
            users_per_page: 25,
            users_per_lookup: 500,
            unread_count_max: 100
        }
    }
}
