use std::collections::HashMap;

use serde::Deserialize;

const TRADEMARK: char = '\u{2122}';
const REGISTERED: char = '\u{00ae}';
const COPYRIGHT: char = '\u{00a9}';

// Data Types //

/// What one game is shown as
#[derive(Deserialize)]
pub struct Mapping {
    /// Name shown instead of what Xbox reports
    pub name: String,

    /// Rich presence prefixes and what each is shown as
    #[serde(default)]
    pub activity: HashMap<String, String>
}

/// Mappings by lookup key
pub type Mappings = HashMap<String, Mapping>;

// Public Methods //

/// Read a mapping file, returns empty map when it cannot be read or parsed
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

/// Name with excessive characters stripped
pub fn remove_excessive(name: &str) -> String {
    let stripped: String = name
        .chars()
        .filter(|character| !matches!(character, &TRADEMARK | &REGISTERED | &COPYRIGHT ))
        .collect();

    stripped.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Rich presence with a listed prefix replaced, and what follows it in parens
pub fn replace_prefix(prefixes: &HashMap<String, String>, activity: &str) -> String {
    for (prefix, shown) in prefixes {
        if !activity.starts_with(prefix) {
            continue;
        }

        let after = &activity[prefix.len()..];
        let rest = after.trim_start_matches([':', '-', ' ']);

        return match rest.is_empty() {
            true => shown.clone(),
            false => format!("{shown} ({rest})")
        };
    }

    activity.to_string()
}

/// Key a name is matched on
/// Lowercase, spaces as underscores, punctuation dropped
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
