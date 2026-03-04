use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

/// A GraphQL request ready to be serialised as JSON and POSTed.
#[derive(Debug, Clone)]
pub struct GraphqlQuery {
    pub url: String,
    pub query: String,
    pub variables: Option<Value>,
    pub operation_name: Option<String>,
    pub headers: HashMap<String, String>,
}

/// The JSON body we send to the GraphQL endpoint.
#[derive(Serialize)]
struct GraphqlBody<'a> {
    query: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<&'a Value>,
    #[serde(rename = "operationName", skip_serializing_if = "Option::is_none")]
    operation_name: Option<&'a str>,
}

impl GraphqlQuery {
    /// Build a new GraphQL query for the given endpoint.
    pub fn new(url: impl Into<String>, query: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            query: query.into(),
            variables: None,
            operation_name: None,
            headers: HashMap::new(),
        }
    }

    /// Attach JSON variables.
    pub fn with_variables(mut self, vars: Value) -> Self {
        self.variables = Some(vars);
        self
    }

    /// Attach an operation name.
    pub fn with_operation(mut self, op: impl Into<String>) -> Self {
        self.operation_name = Some(op.into());
        self
    }

    /// Add a request header.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Serialise this query into the standard GraphQL JSON request body.
    pub fn to_body_json(&self) -> anyhow::Result<String> {
        let body = GraphqlBody {
            query: &self.query,
            variables: self.variables.as_ref(),
            operation_name: self.operation_name.as_deref(),
        };
        Ok(serde_json::to_string(&body)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_graphql_query_serialization_basic() {
        let q = GraphqlQuery::new("http://api", "query { users { id } }");
        let json_str = q.to_body_json().unwrap();

        let parsed: Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["query"], "query { users { id } }");
        assert!(parsed.get("variables").is_none());
        assert!(parsed.get("operationName").is_none());
    }

    #[test]
    fn test_graphql_query_serialization_full() {
        let q = GraphqlQuery::new("http://api", "query GetUser { user { name } }")
            .with_variables(json!({"id": 123}))
            .with_operation("GetUser")
            .with_header("Auth", "secret");

        let json_str = q.to_body_json().unwrap();
        let parsed: Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed["query"], "query GetUser { user { name } }");
        assert_eq!(parsed["variables"]["id"], 123);
        assert_eq!(parsed["operationName"], "GetUser");

        assert_eq!(q.headers.get("Auth").unwrap(), "secret");
    }
}
