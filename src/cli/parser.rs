use clap::{Parser, Subcommand};

/// reqly — a terminal-based HTTP / GraphQL / WebSocket API client.
#[derive(Parser, Debug)]
#[command(
    name = "reqly",
    version = "0.2.0",
    about = "Terminal-native API client: HTTP, GraphQL, WebSocket",
    long_about = None,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    // ── HTTP methods ──────────────────────────────────────────────────────────
    #[command(name = "GET", alias = "get", about = "Send a GET request")]
    Get {
        url: String,
        #[arg(short = 'H', long = "header", value_name = "HEADER")]
        headers: Vec<String>,
    },
    #[command(name = "POST", alias = "post", about = "Send a POST request")]
    Post {
        url: String,
        #[arg(short = 'H', long = "header", value_name = "HEADER")]
        headers: Vec<String>,
        #[arg(short = 'd', long = "data", value_name = "BODY")]
        body: Option<String>,
    },
    #[command(name = "PUT", alias = "put", about = "Send a PUT request")]
    Put {
        url: String,
        #[arg(short = 'H', long = "header", value_name = "HEADER")]
        headers: Vec<String>,
        #[arg(short = 'd', long = "data", value_name = "BODY")]
        body: Option<String>,
    },
    #[command(name = "PATCH", alias = "patch", about = "Send a PATCH request")]
    Patch {
        url: String,
        #[arg(short = 'H', long = "header", value_name = "HEADER")]
        headers: Vec<String>,
        #[arg(short = 'd', long = "data", value_name = "BODY")]
        body: Option<String>,
    },
    #[command(name = "DELETE", alias = "delete", about = "Send a DELETE request")]
    Delete {
        url: String,
        #[arg(short = 'H', long = "header", value_name = "HEADER")]
        headers: Vec<String>,
        #[arg(short = 'd', long = "data", value_name = "BODY")]
        body: Option<String>,
    },
    #[command(name = "HEAD", alias = "head", about = "Send a HEAD request")]
    Head {
        url: String,
        #[arg(short = 'H', long = "header", value_name = "HEADER")]
        headers: Vec<String>,
    },
    #[command(name = "OPTIONS", alias = "options", about = "Send an OPTIONS request")]
    Options {
        url: String,
        #[arg(short = 'H', long = "header", value_name = "HEADER")]
        headers: Vec<String>,
    },

    // ── GraphQL ───────────────────────────────────────────────────────────────
    /// Send a GraphQL query or mutation.
    ///
    /// Example:
    ///   reqly graphql https://api.com/graphql -q 'query { users { id name } }'
    #[command(
        name = "graphql",
        alias = "gql",
        about = "Send a GraphQL query/mutation"
    )]
    Graphql {
        /// GraphQL endpoint URL
        url: String,
        /// GraphQL query string
        #[arg(short = 'q', long = "query", value_name = "QUERY")]
        query: String,
        /// JSON variables object
        #[arg(short = 'v', long = "vars", value_name = "JSON")]
        variables: Option<String>,
        /// Operation name
        #[arg(short = 'o', long = "op", value_name = "NAME")]
        operation: Option<String>,
        /// Request headers (Key: Value)
        #[arg(short = 'H', long = "header", value_name = "HEADER")]
        headers: Vec<String>,
    },

    // ── WebSocket ─────────────────────────────────────────────────────────────
    /// Connect to a WebSocket and start an interactive session.
    ///
    /// Example:
    ///   reqly ws ws://localhost:8080/socket
    #[command(
        name = "ws",
        alias = "websocket",
        about = "Connect to a WebSocket endpoint"
    )]
    Ws {
        /// WebSocket URL (ws:// or wss://)
        url: String,
        /// Request headers (Key: Value)
        #[arg(short = 'H', long = "header", value_name = "HEADER")]
        headers: Vec<String>,
        /// Pretty-print JSON messages from the server
        #[arg(long = "json")]
        json: bool,
        /// Send periodic pings (every 30 s)
        #[arg(long = "ping")]
        ping: bool,
    },
}

// ─── Parsed HTTP request (unchanged) ─────────────────────────────────────────

/// Parsed HTTP request parameters.
pub struct ParsedRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<String>,
    pub body: Option<String>,
}

impl ParsedRequest {
    pub fn from_command(command: &Commands) -> Option<Self> {
        match command {
            Commands::Get { url, headers } => Some(ParsedRequest {
                method: "GET".into(),
                url: url.clone(),
                headers: headers.clone(),
                body: None,
            }),
            Commands::Post { url, headers, body } => Some(ParsedRequest {
                method: "POST".into(),
                url: url.clone(),
                headers: headers.clone(),
                body: body.clone(),
            }),
            Commands::Put { url, headers, body } => Some(ParsedRequest {
                method: "PUT".into(),
                url: url.clone(),
                headers: headers.clone(),
                body: body.clone(),
            }),
            Commands::Patch { url, headers, body } => Some(ParsedRequest {
                method: "PATCH".into(),
                url: url.clone(),
                headers: headers.clone(),
                body: body.clone(),
            }),
            Commands::Delete { url, headers, body } => Some(ParsedRequest {
                method: "DELETE".into(),
                url: url.clone(),
                headers: headers.clone(),
                body: body.clone(),
            }),
            Commands::Head { url, headers } => Some(ParsedRequest {
                method: "HEAD".into(),
                url: url.clone(),
                headers: headers.clone(),
                body: None,
            }),
            Commands::Options { url, headers } => Some(ParsedRequest {
                method: "OPTIONS".into(),
                url: url.clone(),
                headers: headers.clone(),
                body: None,
            }),
            _ => None, // GraphQL and WebSocket are handled elsewhere
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsed_request_from_get_command() {
        let cmd = Commands::Get {
            url: "https://api.test".into(),
            headers: vec!["Accept: application/json".into()],
        };

        let parsed = ParsedRequest::from_command(&cmd).unwrap();

        assert_eq!(parsed.method, "GET");
        assert_eq!(parsed.url, "https://api.test");
        assert_eq!(parsed.headers.len(), 1);
        assert_eq!(parsed.headers[0], "Accept: application/json");
        assert!(parsed.body.is_none());
    }

    #[test]
    fn test_parsed_request_from_post_command() {
        let cmd = Commands::Post {
            url: "https://api.test".into(),
            headers: vec![],
            body: Some(r#"{"hello":"world"}"#.into()),
        };

        let parsed = ParsedRequest::from_command(&cmd).unwrap();

        assert_eq!(parsed.method, "POST");
        assert_eq!(parsed.url, "https://api.test");
        assert!(parsed.headers.is_empty());
        assert_eq!(parsed.body.unwrap(), r#"{"hello":"world"}"#);
    }

    #[test]
    fn test_parsed_request_ignores_non_http() {
        let cmd = Commands::Ws {
            url: "wss://test".into(),
            headers: vec![],
            json: false,
            ping: false,
        };

        let parsed = ParsedRequest::from_command(&cmd);
        assert!(parsed.is_none());
    }
}
