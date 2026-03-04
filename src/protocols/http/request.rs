use std::collections::HashMap;

/// Represents a single HTTP request to be sent.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl HttpRequest {
    pub fn new(method: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            url: url.into(),
            headers: HashMap::new(),
            body: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    #[allow(dead_code)]
    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_request_builder() {
        let req = HttpRequest::new("POST", "https://api.example.com")
            .with_header("Accept", "application/json")
            .with_header("Authorization", "Bearer token123")
            .with_body(r#"{"key":"value"}"#);

        assert_eq!(req.method, "POST");
        assert_eq!(req.url, "https://api.example.com");
        assert_eq!(req.headers.get("Accept").unwrap(), "application/json");
        assert_eq!(req.headers.get("Authorization").unwrap(), "Bearer token123");
        assert_eq!(req.body.unwrap(), r#"{"key":"value"}"#);
    }

    #[test]
    fn test_http_request_new() {
        let req = HttpRequest::new("GET", "https://example.com");
        assert_eq!(req.method, "GET");
        assert_eq!(req.url, "https://example.com");
        assert!(req.headers.is_empty());
        assert!(req.body.is_none());
    }
}
