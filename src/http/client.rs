use anyhow::{Context, Result};
use reqwest::{
    Client, Method,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Instant;

use super::request::HttpRequest;
use super::response::HttpResponse;

/// A thin async HTTP client wrapping `reqwest`.
pub struct HttpClient {
    inner: Client,
}

impl HttpClient {
    /// Create a new `HttpClient`.
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("reqly/0.1.0")
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self { inner: client })
    }

    /// Send an `HttpRequest` and return an `HttpResponse`.
    pub async fn send(&self, req: HttpRequest) -> Result<HttpResponse> {
        let method = Method::from_str(&req.method)
            .with_context(|| format!("Unknown HTTP method: {}", req.method))?;

        let mut header_map = HeaderMap::new();
        for (k, v) in &req.headers {
            let name =
                HeaderName::from_str(k).with_context(|| format!("Invalid header name: {k}"))?;
            let value = HeaderValue::from_str(v)
                .with_context(|| format!("Invalid header value for {k}"))?;
            header_map.insert(name, value);
        }

        let mut builder = self.inner.request(method, &req.url).headers(header_map);

        if let Some(body) = req.body.clone() {
            builder = builder.body(body);
        }

        let start = Instant::now();
        let response = builder
            .send()
            .await
            .with_context(|| format!("Failed to connect to {}", req.url))?;
        let duration_ms = start.elapsed().as_millis();

        let status = response.status();
        let status_code = status.as_u16();
        let status_text = status.canonical_reason().unwrap_or("Unknown").to_string();

        let mut headers: HashMap<String, String> = HashMap::new();
        for (k, v) in response.headers() {
            if let Ok(val) = v.to_str() {
                headers.insert(k.as_str().to_lowercase(), val.to_string());
            }
        }

        let body = response
            .text()
            .await
            .context("Failed to read response body")?;

        Ok(HttpResponse {
            status: status_code,
            status_text,
            headers,
            body,
            duration_ms,
        })
    }
}
