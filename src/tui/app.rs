use crate::protocols::graphql::GraphqlResponse;
use crate::protocols::http::HttpResponse;
use crate::protocols::websocket::ConnectionStatus;

// ─── Protocol Mode ────────────────────────────────────────────────────────────

/// Which protocol the TUI is currently operating in.
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolMode {
    Http,
    GraphQL,
    WebSocket,
}

impl ProtocolMode {
    #[allow(dead_code)]
    pub fn label(&self) -> &str {
        match self {
            ProtocolMode::Http => "HTTP",
            ProtocolMode::GraphQL => "GraphQL",
            ProtocolMode::WebSocket => "WebSocket",
        }
    }

    pub fn next(&self) -> ProtocolMode {
        match self {
            ProtocolMode::Http => ProtocolMode::GraphQL,
            ProtocolMode::GraphQL => ProtocolMode::WebSocket,
            ProtocolMode::WebSocket => ProtocolMode::Http,
        }
    }
}

// ─── Focused Panel ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum FocusedPanel {
    // HTTP panels
    Method,
    Url,
    Headers,
    Body,
    Response,
    // GraphQL panels
    GqlEndpoint,
    GqlQuery,
    GqlVariables,
    GqlResponse,
    // WebSocket panels
    WsUrl,
    WsInput,
    WsMessages,
}

// ─── App State ────────────────────────────────────────────────────────────────

pub struct App {
    // ── Protocol selector
    pub protocol: ProtocolMode,

    // ── HTTP state (same as before)
    pub method: String,
    pub url: String,
    pub headers_raw: String,
    pub body_raw: String,
    pub response: Option<Result<HttpResponse, String>>,

    // ── GraphQL state
    pub gql_endpoint: String,
    pub gql_query: String,
    pub gql_variables: String,
    pub gql_response: Option<Result<GraphqlResponse, String>>,

    // ── WebSocket state
    pub ws_url: String,
    pub ws_status: ConnectionStatus,
    pub ws_messages: Vec<String>,
    pub ws_input: String,

    // ── Shared
    pub focused: FocusedPanel,
    pub is_loading: bool,
    #[allow(dead_code)]
    pub should_quit: bool,
    pub cursor_pos: usize,
    pub response_scroll: u16,
    pub gql_response_scroll: u16,
}

impl App {
    pub fn new() -> Self {
        Self {
            protocol: ProtocolMode::Http,
            // HTTP
            method: "GET".to_string(),
            url: String::new(),
            headers_raw: String::new(),
            body_raw: String::new(),
            response: None,
            // GraphQL
            gql_endpoint: String::new(),
            gql_query: String::new(),
            gql_variables: String::new(),
            gql_response: None,
            // WebSocket
            ws_url: String::new(),
            ws_status: ConnectionStatus::Disconnected,
            ws_messages: Vec::new(),
            ws_input: String::new(),
            // Shared
            focused: FocusedPanel::Url,
            is_loading: false,
            should_quit: false,
            cursor_pos: 0,
            response_scroll: 0,
            gql_response_scroll: 0,
        }
    }

    // ── Protocol cycling ──────────────────────────────────────────────────────

    pub fn cycle_protocol(&mut self) {
        self.protocol = self.protocol.next();
        self.focused = match self.protocol {
            ProtocolMode::Http => FocusedPanel::Url,
            ProtocolMode::GraphQL => FocusedPanel::GqlEndpoint,
            ProtocolMode::WebSocket => FocusedPanel::WsUrl,
        };
        self.cursor_pos = 0;
    }

    // ── HTTP helpers ──────────────────────────────────────────────────────────

    pub fn cycle_method(&mut self) {
        self.method = match self.method.as_str() {
            "GET" => "POST",
            "POST" => "PUT",
            "PUT" => "PATCH",
            "PATCH" => "DELETE",
            "DELETE" => "HEAD",
            "HEAD" => "OPTIONS",
            _ => "GET",
        }
        .to_string();
    }

    // ── Focus navigation ──────────────────────────────────────────────────────

    pub fn focus_next(&mut self) {
        self.focused = match (&self.protocol, &self.focused) {
            (ProtocolMode::Http, FocusedPanel::Method) => FocusedPanel::Url,
            (ProtocolMode::Http, FocusedPanel::Url) => FocusedPanel::Headers,
            (ProtocolMode::Http, FocusedPanel::Headers) => FocusedPanel::Body,
            (ProtocolMode::Http, FocusedPanel::Body) => FocusedPanel::Response,
            (ProtocolMode::Http, FocusedPanel::Response) => FocusedPanel::Method,
            (ProtocolMode::GraphQL, FocusedPanel::GqlEndpoint) => FocusedPanel::GqlQuery,
            (ProtocolMode::GraphQL, FocusedPanel::GqlQuery) => FocusedPanel::GqlVariables,
            (ProtocolMode::GraphQL, FocusedPanel::GqlVariables) => FocusedPanel::GqlResponse,
            (ProtocolMode::GraphQL, FocusedPanel::GqlResponse) => FocusedPanel::GqlEndpoint,
            (ProtocolMode::WebSocket, FocusedPanel::WsUrl) => FocusedPanel::WsInput,
            (ProtocolMode::WebSocket, FocusedPanel::WsInput) => FocusedPanel::WsMessages,
            (ProtocolMode::WebSocket, FocusedPanel::WsMessages) => FocusedPanel::WsUrl,
            _ => self.focused.clone(),
        };
        self.cursor_pos = self.active_text().len();
    }

    pub fn focus_prev(&mut self) {
        self.focused = match (&self.protocol, &self.focused) {
            (ProtocolMode::Http, FocusedPanel::Method) => FocusedPanel::Response,
            (ProtocolMode::Http, FocusedPanel::Url) => FocusedPanel::Method,
            (ProtocolMode::Http, FocusedPanel::Headers) => FocusedPanel::Url,
            (ProtocolMode::Http, FocusedPanel::Body) => FocusedPanel::Headers,
            (ProtocolMode::Http, FocusedPanel::Response) => FocusedPanel::Body,
            (ProtocolMode::GraphQL, FocusedPanel::GqlEndpoint) => FocusedPanel::GqlResponse,
            (ProtocolMode::GraphQL, FocusedPanel::GqlQuery) => FocusedPanel::GqlEndpoint,
            (ProtocolMode::GraphQL, FocusedPanel::GqlVariables) => FocusedPanel::GqlQuery,
            (ProtocolMode::GraphQL, FocusedPanel::GqlResponse) => FocusedPanel::GqlVariables,
            (ProtocolMode::WebSocket, FocusedPanel::WsUrl) => FocusedPanel::WsMessages,
            (ProtocolMode::WebSocket, FocusedPanel::WsInput) => FocusedPanel::WsUrl,
            (ProtocolMode::WebSocket, FocusedPanel::WsMessages) => FocusedPanel::WsInput,
            _ => self.focused.clone(),
        };
        self.cursor_pos = self.active_text().len();
    }

    // ── Text editing ──────────────────────────────────────────────────────────

    pub fn active_text(&self) -> &str {
        match &self.focused {
            FocusedPanel::Url => &self.url,
            FocusedPanel::Headers => &self.headers_raw,
            FocusedPanel::Body => &self.body_raw,
            FocusedPanel::GqlEndpoint => &self.gql_endpoint,
            FocusedPanel::GqlQuery => &self.gql_query,
            FocusedPanel::GqlVariables => &self.gql_variables,
            FocusedPanel::WsUrl => &self.ws_url,
            FocusedPanel::WsInput => &self.ws_input,
            _ => "",
        }
    }

    pub fn active_text_mut(&mut self) -> Option<&mut String> {
        match &self.focused {
            FocusedPanel::Url => Some(&mut self.url),
            FocusedPanel::Headers => Some(&mut self.headers_raw),
            FocusedPanel::Body => Some(&mut self.body_raw),
            FocusedPanel::GqlEndpoint => Some(&mut self.gql_endpoint),
            FocusedPanel::GqlQuery => Some(&mut self.gql_query),
            FocusedPanel::GqlVariables => Some(&mut self.gql_variables),
            FocusedPanel::WsUrl => Some(&mut self.ws_url),
            FocusedPanel::WsInput => Some(&mut self.ws_input),
            _ => None,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        let pos = self.cursor_pos;
        if let Some(buf) = self.active_text_mut() {
            if pos <= buf.len() {
                buf.insert(pos, c);
            }
        }
        self.cursor_pos += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        let pos = self.cursor_pos - 1;
        if let Some(buf) = self.active_text_mut() {
            if pos < buf.len() {
                buf.remove(pos);
            }
        }
        self.cursor_pos = self.cursor_pos.saturating_sub(1);
    }

    pub fn move_cursor_left(&mut self) {
        self.cursor_pos = self.cursor_pos.saturating_sub(1);
    }

    pub fn move_cursor_right(&mut self) {
        let text = self.active_text();
        if self.cursor_pos < text.len() {
            if let Some(c) = text[self.cursor_pos..].chars().next() {
                self.cursor_pos += c.len_utf8();
            } else {
                self.cursor_pos += 1;
            }
        }
    }

    pub fn move_cursor_up(&mut self) {
        let text = self.active_text();
        if text.is_empty() || self.cursor_pos == 0 {
            return;
        }
        let pos = self.cursor_pos;
        let current_line_start = text[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let col = text[current_line_start..pos].chars().count();

        if current_line_start == 0 {
            self.cursor_pos = 0;
            return;
        }

        let prev_line_start = text[..current_line_start - 1]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let prev_line_text = &text[prev_line_start..current_line_start - 1];
        let prev_chars: Vec<(usize, char)> = prev_line_text.char_indices().collect();

        let new_col = std::cmp::min(col, prev_chars.len());
        if new_col == prev_chars.len() {
            self.cursor_pos = current_line_start - 1;
        } else {
            self.cursor_pos = prev_line_start + prev_chars[new_col].0;
        }
    }

    pub fn move_cursor_down(&mut self) {
        let text = self.active_text();
        if text.is_empty() || self.cursor_pos == text.len() {
            return;
        }
        let pos = self.cursor_pos;
        let current_line_start = text[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let col = text[current_line_start..pos].chars().count();

        if let Some(idx) = text[pos..].find('\n') {
            let next_line_start = pos + idx + 1;
            let next_line_text = if let Some(end_idx) = text[next_line_start..].find('\n') {
                &text[next_line_start..next_line_start + end_idx]
            } else {
                &text[next_line_start..]
            };

            let next_chars: Vec<(usize, char)> = next_line_text.char_indices().collect();
            let new_col = std::cmp::min(col, next_chars.len());

            if new_col == next_chars.len() {
                self.cursor_pos = next_line_start + next_line_text.len();
            } else {
                self.cursor_pos = next_line_start + next_chars[new_col].0;
            }
        } else {
            self.cursor_pos = text.len();
        }
    }

    /// Add a message to the WebSocket log.
    pub fn push_ws_message(&mut self, msg: String) {
        self.ws_messages.push(msg);
    }
}
