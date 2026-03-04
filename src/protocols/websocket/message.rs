/// Represents a single WebSocket message frame.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum WsMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping,
    Pong,
    Close,
}

impl WsMessage {
    /// Format the message for display in the terminal.
    pub fn display(&self, pretty_json: bool) -> String {
        match self {
            WsMessage::Text(s) => {
                if pretty_json {
                    serde_json::from_str::<serde_json::Value>(s)
                        .ok()
                        .and_then(|v| serde_json::to_string_pretty(&v).ok())
                        .unwrap_or_else(|| s.clone())
                } else {
                    s.clone()
                }
            }
            WsMessage::Binary(b) => {
                let hex: Vec<String> = b.iter().map(|byte| format!("{byte:02X}")).collect();
                format!("[binary {} bytes] {}", b.len(), hex.join(" "))
            }
            WsMessage::Ping => "[ping]".to_string(),
            WsMessage::Pong => "[pong]".to_string(),
            WsMessage::Close => "[connection closed]".to_string(),
        }
    }
}
