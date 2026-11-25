//! The signed-in user's profile

use std::sync::Mutex;

use types::users::UserProfile;

use crate::api::{self, ErrorMessage};
use crate::events;
use crate::state::session;

const NOT_SIGNED_IN: &str = "Not signed in";
const LOCK_FAILED: &str = "profile should not be left locked by a crash";

static PROFILE: Mutex<Option<UserProfile>> = Mutex::new(None);


/// Load the signed-in account's profile from its server
pub async fn load() -> Result<(), ErrorMessage> {
    let account = match session::active_account() {
        Some(account) => account,
        None => return Err(NOT_SIGNED_IN.to_string()),
    };

    let loaded = api::me(&account.server, &account.session).await?;
    *PROFILE.lock().expect(LOCK_FAILED) = Some(loaded);
    events::PROFILE_CHANGED.publish();
    Ok(())
}

/// Determine how subscribed events are handled
pub fn subscribe() {
    events::SIGNED_OUT.subscribe(clear);
}

/// Get the loaded profile, if any
pub fn get() -> Option<UserProfile> {
    PROFILE.lock().expect(LOCK_FAILED).clone()
}

fn clear() {
    PROFILE.lock().expect(LOCK_FAILED).take();
    events::PROFILE_CHANGED.publish();
}
