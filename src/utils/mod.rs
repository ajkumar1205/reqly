/// Parse a "Key: Value" header string into a (String, String) tuple.
pub fn parse_header(raw: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = raw.splitn(2, ':').collect();
    if parts.len() == 2 {
        Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
    } else {
        None
    }
}
