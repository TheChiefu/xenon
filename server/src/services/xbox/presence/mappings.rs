//! Maps game name and activities to user defined versions in mapped JSON file.
//! For example:
//! - "Halo: The Master Chief Collection" can be mapped to "Halo MCC"
//! - "Halo R: Campaign" can be mapped to "Reach: Campaign"
//! To avoid repeated words in concat names like "Halo: Halo R: Campaign"
//! Automatically removes extra characters like trademarks to clean up names

use std::collections::HashMap;

use serde::Deserialize;

const TRADEMARK: char = '\u{2122}';
const REGISTERED: char = '\u{00ae}';
const COPYRIGHT: char = '\u{00a9}';

// DATA STRUCTS //

/// Game mapping loaded from mapping file
/// - display name for title
/// - rich presence prefixes and their replacements
#[derive(Deserialize)]
pub struct Mapping {
    pub name: String,

    #[serde(default)]
    pub activity: HashMap<String, String>
}

/// Mappings by lookup key
pub type Mappings = HashMap<String, Mapping>;

// PUBLIC METHODS //

/// Loads mapping file, empty if file can't be read or parsed
pub fn load(path: &str) -> Mappings {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) => {
            tracing::error!("{path}: {e}");
            return Mappings::new();
        }
    };

    match serde_json::from_str(&text) {
        Ok(mappings) => mappings,
        Err(e) => {
            tracing::error!("{path}: {e}");
            Mappings::new()
        }
    }
}

/// Removes symbols and extra spaces from title name
pub fn clean_title(name: &str) -> String {
    let stripped: String = name
        .chars()
        .filter(|character| !matches!(character, &TRADEMARK | &REGISTERED | &COPYRIGHT ))
        .collect();

    stripped.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Rich presence with a listed prefix replaced, and what follows it in parens
pub fn replace_prefix(prefixes: &HashMap<String, String>, rich_presence: &str) -> String {
    // Example: mapping "Halo R" to "Reach" turns "Halo R: Campaign" into "Reach (Campaign)"
    for (prefix, replacement) in prefixes {
        // Skips mapping unless rich presence starts with mapped prefix (e.g. "Halo R")
        if !rich_presence.starts_with(prefix) {
            continue;
        }

        // Gets text after mapped prefix and separators (ie. "Campaign" from "Halo R: Campaign")
        let suffix = &rich_presence[prefix.len()..];
        let suffix = suffix.trim_start_matches([':', '-', ' ']);

        // Appends the suffix in parentheses when one exists (ie. "Reach (Campaign)")
        return match suffix.is_empty() {
            true => replacement.clone(),
            false => format!("{replacement} ({suffix})")
        };
    }

    // Returns rich presence unchanged if no mapping matches
    rich_presence.to_string()
}

/// Returns lookup key for title name
pub fn lookup_key(name: &str) -> String {
    let mut key = String::with_capacity(name.len());

    for character in name.to_lowercase().chars() {
        if character.is_alphanumeric() {
            key.push(character);
            continue;
        }

        if character == ' ' {
            key.push('_');
        }
    }

    key
}
