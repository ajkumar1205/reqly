use crate::http::HttpResponse;

/// Which pane of the TUI is focused.
#[derive(Debug, Clone, PartialEq)]
pub enum FocusedPanel {
    Method,
    Url,
    Headers,
    Body,
    Response,
}

/// Overall state of the TUI application.
pub struct App {
    pub method: String,
    pub url: String,
    pub headers_raw: String,
    pub body_raw: String,
    pub response: Option<Result<HttpResponse, String>>,
    pub focused: FocusedPanel,
    pub is_loading: bool,
    #[allow(dead_code)]
    pub should_quit: bool,
    /// cursor offset within the currently focused text field
    pub cursor_pos: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            method: "GET".to_string(),
            url: String::new(),
            headers_raw: String::new(),
            body_raw: String::new(),
            response: None,
            focused: FocusedPanel::Url,
            is_loading: false,
            should_quit: false,
            cursor_pos: 0,
        }
    }

    /// Cycle through the supported HTTP methods.
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

    /// Move focus to the next panel in order.
    pub fn focus_next(&mut self) {
        self.focused = match self.focused {
            FocusedPanel::Method => FocusedPanel::Url,
            FocusedPanel::Url => FocusedPanel::Headers,
            FocusedPanel::Headers => FocusedPanel::Body,
            FocusedPanel::Body => FocusedPanel::Response,
            FocusedPanel::Response => FocusedPanel::Method,
        };
        self.cursor_pos = self.active_text().len();
    }

    /// Move focus to the previous panel.
    pub fn focus_prev(&mut self) {
        self.focused = match self.focused {
            FocusedPanel::Method => FocusedPanel::Response,
            FocusedPanel::Url => FocusedPanel::Method,
            FocusedPanel::Headers => FocusedPanel::Url,
            FocusedPanel::Body => FocusedPanel::Headers,
            FocusedPanel::Response => FocusedPanel::Body,
        };
        self.cursor_pos = self.active_text().len();
    }

    /// Return a mutable reference to the text buffer for the focused panel.
    pub fn active_text_mut(&mut self) -> Option<&mut String> {
        match self.focused {
            FocusedPanel::Url => Some(&mut self.url),
            FocusedPanel::Headers => Some(&mut self.headers_raw),
            FocusedPanel::Body => Some(&mut self.body_raw),
            _ => None,
        }
    }

    pub fn active_text(&self) -> &str {
        match self.focused {
            FocusedPanel::Url => &self.url,
            FocusedPanel::Headers => &self.headers_raw,
            FocusedPanel::Body => &self.body_raw,
            _ => "",
        }
    }

    /// Push a char at the current cursor position.
    pub fn insert_char(&mut self, c: char) {
        let pos = self.cursor_pos;
        if let Some(buf) = self.active_text_mut() {
            if pos <= buf.len() {
                buf.insert(pos, c);
            }
        }
        self.cursor_pos += 1;
    }

    /// Delete char before cursor (backspace).
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
        let max = self.active_text().len();
        if self.cursor_pos < max {
            self.cursor_pos += 1;
        }
    }
}
