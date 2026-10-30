//! Format checks run before a value reaches the database.

use crate::config;
use crate::error::{AppError, Result};
use crate::types::messages::SearchRequest;
use crate::types::preferences::RoomListItem;
use crate::types::time::DAY;

/// How many registrations a code covers when the request names no count.
pub const REGISTRATION_CODE_DEFAULT_MAX_USES: i64 = 1;

/// How long a registration code lasts when the request names no lifetime.
pub const REGISTRATION_CODE_LIFETIME_MS: i64 = DAY * 7;

pub const REGISTRATION_CODE_DEFAULT_LENGTH: usize = 12;
const REGISTRATION_CODE_LENGTH_MIN: usize = 4;
const REGISTRATION_CODE_LENGTH_MAX: usize = 64;

/// Filesystem limit on one path component.
const FILE_NAME_MAX: usize = 255;

/// Shortest text the trigram search index can match
const SEARCH_TEXT_MIN: usize = 3;

/// Check a login name's length and character set
pub fn username(name: &str) -> Result<()> {
    let len = name.chars().count();
    let min = config::get().lengths.username_min;
    let max = config::get().lengths.username_max;

    // Enforce length limits
    if len < min || len > max {
        return Err(AppError::Validation(
            format!("username error: must be between {min} and {max} characters")
        ));
    }

    // Enforce username restrictions
    for c in name.chars() {
        let allowed = c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-';
        if !allowed {
            return Err(AppError::Validation(
                "username error: may only contain lowercase, digits, underscores and hyphens"
                    .to_string()
            ));
        }
    }

    Ok(())
}

/// Check a display name's length
pub fn display_name(name: &str) -> Result<()> {
    let len = name.chars().count();
    let min = config::get().lengths.display_name_min;
    let max = config::get().lengths.display_name_max;

    if len < min || len > max {
        return Err(AppError::Validation(
            format!("display name error: must be between {min} and {max} characters")
        ));
    }

    Ok(())
}

/// Check a profile description's length
pub fn profile_description(text: &str) -> Result<()> {
    let len = text.chars().count();
    let max = config::get().lengths.profile_description_max;

    if len > max {
        return Err(AppError::Validation(
            format!("description error: outside of max character limit ({max})")
        ));
    }

    Ok(())
}

/// Check a password's length
pub fn password(password: &str) -> Result<()> {
    let len = password.chars().count();
    let min = config::get().lengths.password_min;
    let max = config::get().lengths.password_max;

    if len < min || len > max {
        return Err(AppError::Validation(
            format!("password error: outside of available range ({min} - {max})")
        ));
    }

    Ok(())
}

/// Check the settings a registration code is created with
pub fn registration_code_params(max_uses: i64, lifetime: i64, length: usize) -> Result<()> {

    if max_uses < 1 {
        return Err(AppError::Validation(
            "registration code error: max uses must be at least 1".to_string()
        ));
    }

    if lifetime < 1 {
        return Err(AppError::Validation(
            "registration code error: lifetime must be at least 1 ms".to_string()
        ));
    }

    let min = REGISTRATION_CODE_LENGTH_MIN;
    let max = REGISTRATION_CODE_LENGTH_MAX;

    if length < min || length > max {
        return Err(AppError::Validation(
            format!("registration code error: length must be between {min} and {max}")
        ));
    }

    Ok(())
}

/// Check a room name's length and returns it as stored
/// A name of nothing but spaces is stored as empty
pub fn room_name(name: &str) -> Result<&str> {
    let len = name.chars().count();
    let max = config::get().lengths.room_name_max;

    if len > max {
        return Err(AppError::Validation(
            format!("room name error: Name longer than character limit [{max}]")
        ));
    }

    if name.trim().is_empty() {
        return Ok("");
    }

    Ok(name)
}

/// Return a typed registration code in its stored form, trimmed and uppercase
pub fn registration_code(code: &str) -> String {
    code.trim().to_ascii_uppercase()
}

/// Strip any directory a client sent and returns the name alone
pub fn file_name(path: &str) -> Result<String> {

    // Find separators in path (client's OS is unknown match both slashes)
    let index = match path.rfind(['/', '\\']) {
        Some(i) => i + 1,
        None => 0
    };
    let name = &path[index..];

    // Strip any folder leading dots (ie up dir / same dir)
    if name.is_empty() || name == "." || name == ".." {
        return Err(AppError::Validation(
            format!("file name error: [{path}] names a directory")
        ));
    }

    // If remaining filename is larger than OS limit, error
    let len = name.chars().count();
    if len > FILE_NAME_MAX {
        return Err(AppError::Validation(
            format!("file name error: longer than character limit [{FILE_NAME_MAX}]")
        ));
    }

    Ok(name.to_string())
}

/// Check a message body's length
pub fn message_body(content: &str) -> Result<()> {
    let len = content.chars().count();
    let max = config::get().lengths.message_body_max;
    if len > max {
        return Err(AppError::Validation(
            format!("message error: outside of max character limit ({max})")
        ));
    }

    Ok(())
}

/// Check a search has at least one filter and that each filter is usable
pub fn search(request: &SearchRequest) -> Result<()> {
    let has_filter = request.text.is_some()
        || request.file.is_some()
        || request.room.is_some()
        || request.author.is_some()
        || request.from.is_some()
        || request.to.is_some();

    if !has_filter {
        return Err(AppError::Validation(
            "search error: needs at least one filter".to_string()
        ));
    }

    // Message body text
    if let Some(text) = &request.text {
        search_length("text", text, config::get().lengths.message_body_max)?;
    }

    // File name text
    if let Some(file) = &request.file {
        search_length("file", file, FILE_NAME_MAX)?;
    }

    // Page number
    if request.page < 1 {
        return Err(AppError::Validation(
            "search error: page must be at least 1".to_string()
        ));
    }

    // Date range order
    if let (Some(from), Some(to)) = (request.from, request.to) {
        if from >= to {
            return Err(AppError::Validation(
                "search error: date range must start before it ends".to_string()
            ));
        }
    }

    Ok(())
}

/// Check searched text against the trigram minimum and its field's maximum
fn search_length(field: &str, text: &str, max: usize) -> Result<()> {
    let len = text.chars().count();
    let min = SEARCH_TEXT_MIN;

    if len < min || len > max {
        return Err(AppError::Validation(
            format!("search error: {field} must be between {min} and {max} characters")
        ));
    }

    Ok(())
}

/// Check that a room list layout is a JSON array of RoomListItem
pub fn room_layout(layout: &str) -> Result<()> {
    let parsed: serde_json::Result<Vec<RoomListItem>> = serde_json::from_str(layout);

    if let Err(e) = parsed {
        return Err(AppError::Validation(format!("room layout error: {e}")));
    }

    Ok(())
}
