use std::collections::HashMap;

/// Represents a received HTTP response.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub duration_ms: u128,
}

impl HttpResponse {
    pub fn is_json(&self) -> bool {
        self.headers
            .get("content-type")
            .map(|v| v.contains("application/json"))
            .unwrap_or(false)
    }
}
