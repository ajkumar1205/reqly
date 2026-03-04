use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite::{Message, client::IntoClientRequest, http::HeaderValue},
};

use super::message::WsMessage;

/// Async WebSocket client built on `tokio-tungstenite`.
pub struct WebSocketClient;

impl WebSocketClient {
    /// Run an interactive CLI WebSocket session.
    ///
    /// Connects to `url`, then:
    ///  - Spawns a task to print incoming messages.
    ///  - Reads lines from `stdin` and sends them as Text frames.
    ///  - Exits when the connection closes or the user presses Ctrl+C.
    pub async fn run_interactive(
        url: &str,
        headers: &HashMap<String, String>,
        pretty_json: bool,
    ) -> Result<()> {
        use colored::Colorize;
        use tokio::io::{AsyncBufReadExt, BufReader};

        // Build the WebSocket request, injecting custom headers
        let mut request = url
            .into_client_request()
            .with_context(|| format!("Invalid WebSocket URL: {url}"))?;

        for (k, v) in headers {
            let name = reqwest::header::HeaderName::from_bytes(k.as_bytes())
                .with_context(|| format!("Invalid header name: {k}"))?;
            let value =
                HeaderValue::from_str(v).with_context(|| format!("Invalid header value: {v}"))?;
            request.headers_mut().insert(name, value);
        }

        println!();
        println!("{}", format!("Connecting to {url}").cyan().bold());

        let (ws_stream, _) = connect_async_tls_with_config(request, None, false, None)
            .await
            .with_context(|| format!("WebSocket handshake failed for {url}"))?;

        println!("{}", format!("Connected to {url}").green().bold());
        println!(
            "{}",
            "Type a message and press ENTER to send. Ctrl+C to quit.".dimmed()
        );
        println!();

        let (mut write, mut read) = ws_stream.split();

        // Spawn a task to receive and print incoming frames
        tokio::spawn(async move {
            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(Message::Text(t)) => {
                        let ws_msg = WsMessage::Text(t.to_string());
                        println!(
                            "{} {}",
                            "Server:".yellow().bold(),
                            ws_msg.display(pretty_json)
                        );
                    }
                    Ok(Message::Binary(b)) => {
                        let ws_msg = WsMessage::Binary(b.to_vec());
                        println!(
                            "{} {}",
                            "Server:".yellow().bold(),
                            ws_msg.display(pretty_json)
                        );
                    }
                    Ok(Message::Ping(_)) => {
                        println!("{}", "[ping received]".dimmed());
                    }
                    Ok(Message::Pong(_)) => {
                        println!("{}", "[pong received]".dimmed());
                    }
                    Ok(Message::Close(_)) => {
                        println!("{}", "Server closed the connection.".red());
                        break;
                    }
                    Err(e) => {
                        eprintln!("{} {}", "error:".red().bold(), e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Read lines from stdin and send as Text frames
        let stdin = tokio::io::stdin();
        let mut lines = BufReader::new(stdin).lines();

        while let Some(line) = lines.next_line().await? {
            let line = line.trim().to_string();
            if line.is_empty() {
                continue;
            }
            write
                .send(Message::Text(line.clone().into()))
                .await
                .context("Failed to send WebSocket message")?;
            println!("{} {}", "Sent:".cyan(), line);
        }

        // Send graceful close
        let _ = write.send(Message::Close(None)).await;
        Ok(())
    }
}
