use serde_json::Value;

/// Pretty-print a JSON string. Returns the original string if parsing fails.
pub fn pretty_print(raw: &str) -> String {
    match serde_json::from_str::<Value>(raw) {
        Ok(value) => serde_json::to_string_pretty(&value).unwrap_or_else(|_| raw.to_string()),
        Err(_) => raw.to_string(),
    }
}

/// Returns true if the string appears to be valid JSON.
#[allow(dead_code)]
pub fn is_json(raw: &str) -> bool {
    serde_json::from_str::<Value>(raw).is_ok()
}
