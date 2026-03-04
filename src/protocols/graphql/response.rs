use serde_json::Value;

/// Result of a GraphQL HTTP round-trip.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GraphqlResponse {
    pub status: u16,
    pub status_text: String,
    pub data: Option<Value>,
    pub errors: Option<Vec<Value>>,
    pub raw_body: String,
    pub duration_ms: u128,
}

impl GraphqlResponse {
    /// True when the GraphQL `errors` array is present and non-empty.
    pub fn has_errors(&self) -> bool {
        self.errors.as_ref().map(|e| !e.is_empty()).unwrap_or(false)
    }

    /// Pretty-print the full response body JSON.
    pub fn pretty_body(&self) -> String {
        serde_json::from_str::<Value>(&self.raw_body)
            .ok()
            .and_then(|v| serde_json::to_string_pretty(&v).ok())
            .unwrap_or_else(|| self.raw_body.clone())
    }
}
