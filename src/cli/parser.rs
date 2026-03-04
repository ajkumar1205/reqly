use clap::{Parser, Subcommand};

/// reqly — a terminal-based HTTP API client.
#[derive(Parser, Debug)]
#[command(
    name = "reqly",
    version = "0.1.0",
    about = "A terminal-based HTTP API client (Postman for the terminal)",
    long_about = None,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Send an HTTP request directly from the CLI.
    #[command(name = "GET", alias = "get", about = "Send a GET request")]
    Get {
        /// Target URL
        url: String,
        /// Request headers in "Key: Value" format (repeat for multiple)
        #[arg(short = 'H', long = "header", value_name = "HEADER")]
        headers: Vec<String>,
    },
    #[command(name = "POST", alias = "post", about = "Send a POST request")]
    Post {
        url: String,
        #[arg(short = 'H', long = "header", value_name = "HEADER")]
        headers: Vec<String>,
        /// Request body (JSON or raw string)
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
}

/// Parsed request parameters ready for use.
pub struct ParsedRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<String>,
    pub body: Option<String>,
}

impl ParsedRequest {
    pub fn from_command(command: Commands) -> Self {
        match command {
            Commands::Get { url, headers } => ParsedRequest {
                method: "GET".into(),
                url,
                headers,
                body: None,
            },
            Commands::Post { url, headers, body } => ParsedRequest {
                method: "POST".into(),
                url,
                headers,
                body,
            },
            Commands::Put { url, headers, body } => ParsedRequest {
                method: "PUT".into(),
                url,
                headers,
                body,
            },
            Commands::Patch { url, headers, body } => ParsedRequest {
                method: "PATCH".into(),
                url,
                headers,
                body,
            },
            Commands::Delete { url, headers, body } => ParsedRequest {
                method: "DELETE".into(),
                url,
                headers,
                body,
            },
            Commands::Head { url, headers } => ParsedRequest {
                method: "HEAD".into(),
                url,
                headers,
                body: None,
            },
            Commands::Options { url, headers } => ParsedRequest {
                method: "OPTIONS".into(),
                url,
                headers,
                body: None,
            },
        }
    }
}
