/// Current state of a WebSocket connection.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

impl ConnectionStatus {
    pub fn label(&self) -> &str {
        match self {
            ConnectionStatus::Disconnected => "Disconnected",
            ConnectionStatus::Connecting => "Connecting…",
            ConnectionStatus::Connected => "Connected ✓",
            ConnectionStatus::Error(_) => "Error",
        }
    }
}

impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionStatus::Error(e) => write!(f, "Error: {e}"),
            other => write!(f, "{}", other.label()),
        }
    }
}
