use anyhow::{Context, Result};
use reqwest::{
    Client,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use std::str::FromStr;
use std::time::Instant;

use super::query::GraphqlQuery;
use super::response::GraphqlResponse;

/// Sends GraphQL requests over HTTP POST using `reqwest`.
pub struct GraphqlClient {
    inner: Client,
}

impl GraphqlClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("reqly/0.1.0")
            .build()
            .context("Failed to build GraphQL HTTP client")?;
        Ok(Self { inner: client })
    }

    /// Execute a GraphQL query against the configured endpoint.
    pub async fn execute(&self, query: &GraphqlQuery) -> Result<GraphqlResponse> {
        let body_json = query.to_body_json()?;

        // Build headers — always set Content-Type
        let mut header_map = HeaderMap::new();
        header_map.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("application/json"),
        );
        for (k, v) in &query.headers {
            let name =
                HeaderName::from_str(k).with_context(|| format!("Invalid header name: {k}"))?;
            let value = HeaderValue::from_str(v)
                .with_context(|| format!("Invalid header value for {k}"))?;
            header_map.insert(name, value);
        }

        let start = Instant::now();
        let response = self
            .inner
            .post(&query.url)
            .headers(header_map)
            .body(body_json)
            .send()
            .await
            .with_context(|| format!("Failed to connect to GraphQL endpoint: {}", query.url))?;
        let duration_ms = start.elapsed().as_millis();

        let status = response.status();
        let status_code = status.as_u16();
        let status_text = status.canonical_reason().unwrap_or("Unknown").to_string();

        let raw_body = response
            .text()
            .await
            .context("Failed to read GraphQL response body")?;

        // Parse the standard GraphQL response shape: { data, errors }
        let parsed: serde_json::Value =
            serde_json::from_str(&raw_body).unwrap_or(serde_json::Value::Null);

        let data = parsed.get("data").cloned();
        let errors = parsed
            .get("errors")
            .and_then(|e| e.as_array())
            .map(|arr| arr.to_vec());

        Ok(GraphqlResponse {
            status: status_code,
            status_text,
            data,
            errors,
            raw_body,
            duration_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_graphql_client_query() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            // GraphqlClient automatically forces application/json content-type
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": { "user": { "name": "Alice" } }
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = GraphqlClient::new().unwrap();
        let query = GraphqlQuery::new(
            format!("{}/graphql", mock_server.uri()),
            "query { user { name } }",
        );

        let res = client.execute(&query).await.unwrap();

        assert_eq!(res.status, 200);
        assert!(!res.has_errors());

        let data = res.data.unwrap();
        assert_eq!(data["user"]["name"], "Alice");
    }
}
