use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::app::{App, FocusedPanel, ProtocolMode};
use crate::formatter;

// ─── Colour palette ───────────────────────────────────────────────────────────
const COLOR_ACCENT: Color = Color::Rgb(80, 200, 200); // teal
const COLOR_DIM: Color = Color::Rgb(100, 100, 120);
const COLOR_FG: Color = Color::Rgb(220, 220, 235);
const COLOR_BG: Color = Color::Rgb(18, 18, 28);
const COLOR_PANEL: Color = Color::Rgb(28, 28, 42);
const COLOR_SUCCESS: Color = Color::Rgb(80, 200, 120);
const COLOR_ERROR: Color = Color::Rgb(220, 80, 80);
const COLOR_METHOD: Color = Color::Rgb(240, 180, 50);
const COLOR_GQL: Color = Color::Rgb(220, 100, 200); // magenta for GraphQL
const COLOR_WS: Color = Color::Rgb(100, 180, 255); // blue for WebSocket

// ─── Main draw entry point ────────────────────────────────────────────────────

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();
    f.render_widget(Block::default().style(Style::default().bg(COLOR_BG)), size);

    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title bar
            Constraint::Length(3), // protocol selector
            Constraint::Min(10),   // protocol-specific content
            Constraint::Length(1), // status bar
        ])
        .split(size);

    draw_title_bar(f, main[0]);
    draw_protocol_tabs(f, app, main[1]);

    match app.protocol {
        ProtocolMode::Http => draw_http_panes(f, app, main[2]),
        ProtocolMode::GraphQL => draw_graphql_panes(f, app, main[2]),
        ProtocolMode::WebSocket => draw_websocket_panes(f, app, main[2]),
    }

    draw_status_bar(f, app, main[3]);
}

// ─── Title bar ────────────────────────────────────────────────────────────────

fn draw_title_bar(f: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " ⚡ reqly ",
            Style::default()
                .fg(COLOR_ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "v0.1.0 — HTTP · GraphQL · WebSocket",
            Style::default().fg(COLOR_DIM),
        ),
    ]))
    .style(Style::default().bg(COLOR_BG));
    f.render_widget(title, area);
}

// ─── Protocol tabs ────────────────────────────────────────────────────────────

fn draw_protocol_tabs(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(12),
            Constraint::Length(14),
            Constraint::Length(16),
            Constraint::Min(0),
        ])
        .split(area);

    // Helper: render a single protocol tab directly to avoid closure lifetime issues
    let render_tab =
        |f: &mut Frame, label: &'static str, color: Color, active: bool, chunk: Rect| {
            let style = if active {
                Style::default()
                    .fg(COLOR_BG)
                    .bg(color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color).bg(COLOR_PANEL)
            };
            let border_style = if active {
                Style::default().fg(color)
            } else {
                Style::default().fg(COLOR_DIM)
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style);
            f.render_widget(Paragraph::new(label).block(block).style(style), chunk);
        };

    render_tab(
        f,
        " HTTP ",
        COLOR_ACCENT,
        app.protocol == ProtocolMode::Http,
        chunks[0],
    );
    render_tab(
        f,
        " GraphQL ",
        COLOR_GQL,
        app.protocol == ProtocolMode::GraphQL,
        chunks[1],
    );
    render_tab(
        f,
        " WebSocket ",
        COLOR_WS,
        app.protocol == ProtocolMode::WebSocket,
        chunks[2],
    );

    let hint = Paragraph::new("  Ctrl+P: switch protocol")
        .style(Style::default().fg(COLOR_DIM).bg(COLOR_BG));
    f.render_widget(hint, chunks[3]);
}

// ─── HTTP panes (identical to v0.1 behaviour) ────────────────────────────────

fn draw_http_panes(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // method + url
            Constraint::Length(7), // headers
            Constraint::Length(7), // body
            Constraint::Min(8),    // response
        ])
        .split(area);

    // Method + URL row
    let row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(10), Constraint::Min(20)])
        .split(chunks[0]);

    let method_block = Block::default()
        .borders(Borders::ALL)
        .border_style(focused_border(app.focused == FocusedPanel::Method))
        .title(Span::styled(" Method ", Style::default().fg(COLOR_DIM)));
    let method_style = Style::default()
        .fg(COLOR_METHOD)
        .bg(COLOR_PANEL)
        .add_modifier(if app.focused == FocusedPanel::Method {
            Modifier::BOLD
        } else {
            Modifier::empty()
        });
    f.render_widget(
        Paragraph::new(app.method.as_str())
            .block(method_block)
            .style(method_style),
        row[0],
    );

    draw_editable_panel(
        f,
        row[1],
        " URL ",
        &app.url,
        app.focused == FocusedPanel::Url,
        COLOR_DIM,
        app.cursor_pos,
    );

    // Headers
    draw_editable_panel(
        f,
        chunks[1],
        " Headers (one per line: Key: Value) ",
        &app.headers_raw,
        app.focused == FocusedPanel::Headers,
        COLOR_DIM,
        app.cursor_pos,
    );

    // Body
    draw_editable_panel(
        f,
        chunks[2],
        " Body (JSON) ",
        &app.body_raw,
        app.focused == FocusedPanel::Body,
        COLOR_DIM,
        app.cursor_pos,
    );

    // Response
    draw_http_response(f, app, chunks[3]);
}

fn draw_http_response(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused == FocusedPanel::Response;

    match &app.response {
        None if app.is_loading => {
            simple_panel(f, area, " Response ", "⏳ Sending…", COLOR_DIM, is_focused);
        }
        None => {
            simple_panel(
                f,
                area,
                " Response ",
                "Press ENTER to send",
                COLOR_DIM,
                is_focused,
            );
        }
        Some(Err(e)) => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_ERROR))
                .title(Span::styled(
                    " Response — Error ",
                    Style::default().fg(COLOR_ERROR),
                ))
                .style(Style::default().bg(COLOR_PANEL));
            f.render_widget(
                Paragraph::new(e.as_str())
                    .block(block)
                    .style(Style::default().fg(COLOR_ERROR))
                    .wrap(Wrap { trim: false }),
                area,
            );
        }
        Some(Ok(resp)) => {
            let status_color = if resp.status < 300 {
                COLOR_SUCCESS
            } else {
                COLOR_ERROR
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(focused_border(is_focused))
                .title(Line::from(vec![
                    Span::styled(" Response  ", Style::default().fg(COLOR_DIM)),
                    Span::styled(
                        format!(
                            "{} {}  {}ms",
                            resp.status, resp.status_text, resp.duration_ms
                        ),
                        Style::default()
                            .fg(status_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]))
                .style(Style::default().bg(COLOR_PANEL));

            let body = if resp.is_json() {
                formatter::pretty_print(&resp.body)
            } else {
                resp.body.clone()
            };

            let mut lines: Vec<Line> = vec![];
            for (k, v) in resp.headers.iter().take(5) {
                lines.push(Line::from(vec![
                    Span::styled(format!("{k}: "), Style::default().fg(COLOR_DIM)),
                    Span::styled(v, Style::default().fg(COLOR_FG)),
                ]));
            }
            lines.push(Line::from(""));
            for raw_line in body.lines() {
                lines.push(Line::from(Span::styled(
                    raw_line,
                    Style::default().fg(COLOR_FG),
                )));
            }

            f.render_widget(
                Paragraph::new(Text::from(lines))
                    .block(block)
                    .wrap(Wrap { trim: false })
                    .scroll((app.response_scroll, 0)),
                area,
            );
        }
    }
}

// ─── GraphQL panes ────────────────────────────────────────────────────────────

fn draw_graphql_panes(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // endpoint URL
            Constraint::Length(10), // query editor
            Constraint::Length(7),  // variables editor
            Constraint::Min(8),     // response
        ])
        .split(area);

    // Endpoint URL
    draw_editable_panel(
        f,
        chunks[0],
        " Endpoint ",
        &app.gql_endpoint,
        app.focused == FocusedPanel::GqlEndpoint,
        COLOR_GQL,
        app.cursor_pos,
    );

    // Query editor
    draw_editable_panel(
        f,
        chunks[1],
        " Query ",
        &app.gql_query,
        app.focused == FocusedPanel::GqlQuery,
        COLOR_GQL,
        app.cursor_pos,
    );

    // Variables editor
    draw_editable_panel(
        f,
        chunks[2],
        " Variables (JSON) ",
        &app.gql_variables,
        app.focused == FocusedPanel::GqlVariables,
        COLOR_GQL,
        app.cursor_pos,
    );

    // Response
    draw_graphql_response(f, app, chunks[3]);
}

fn draw_graphql_response(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused == FocusedPanel::GqlResponse;

    match &app.gql_response {
        None if app.is_loading => {
            simple_panel(f, area, " Response ", "⏳ Sending…", COLOR_DIM, is_focused);
        }
        None => {
            simple_panel(
                f,
                area,
                " Response ",
                "Press ENTER to send the query",
                COLOR_DIM,
                is_focused,
            );
        }
        Some(Err(e)) => {
            let block = error_block(" GraphQL Response — Error ");
            f.render_widget(
                Paragraph::new(e.as_str())
                    .block(block)
                    .style(Style::default().fg(COLOR_ERROR))
                    .wrap(Wrap { trim: false }),
                area,
            );
        }
        Some(Ok(resp)) => {
            let status_color = if resp.has_errors() {
                COLOR_ERROR
            } else {
                COLOR_SUCCESS
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(focused_border_color(is_focused, COLOR_GQL))
                .title(Line::from(vec![
                    Span::styled(" GraphQL Response  ", Style::default().fg(COLOR_DIM)),
                    Span::styled(
                        format!(
                            "{} {}  {}ms",
                            resp.status, resp.status_text, resp.duration_ms
                        ),
                        Style::default()
                            .fg(status_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    if resp.has_errors() {
                        Span::styled(
                            "  ⚠ errors",
                            Style::default()
                                .fg(COLOR_ERROR)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Span::raw("")
                    },
                ]))
                .style(Style::default().bg(COLOR_PANEL));

            let body_str = resp.pretty_body();
            let mut lines: Vec<Line> = vec![];
            for raw_line in body_str.lines() {
                lines.push(Line::from(Span::styled(
                    raw_line,
                    Style::default().fg(COLOR_FG),
                )));
            }

            f.render_widget(
                Paragraph::new(Text::from(lines))
                    .block(block)
                    .wrap(Wrap { trim: false })
                    .scroll((app.gql_response_scroll, 0)),
                area,
            );
        }
    }
}

// ─── WebSocket panes ──────────────────────────────────────────────────────────

fn draw_websocket_panes(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // URL
            Constraint::Length(3), // status
            Constraint::Min(8),    // messages log
            Constraint::Length(3), // input
        ])
        .split(area);

    // URL
    draw_editable_panel(
        f,
        chunks[0],
        " WebSocket URL (ws:// or wss://) ",
        &app.ws_url,
        app.focused == FocusedPanel::WsUrl,
        COLOR_WS,
        app.cursor_pos,
    );

    // Connection status
    use crate::protocols::websocket::ConnectionStatus;
    let (status_label, status_color) = match &app.ws_status {
        ConnectionStatus::Connected => (app.ws_status.to_string(), COLOR_SUCCESS),
        ConnectionStatus::Connecting => (app.ws_status.to_string(), COLOR_METHOD),
        ConnectionStatus::Error(_) => (app.ws_status.to_string(), COLOR_ERROR),
        ConnectionStatus::Disconnected => (app.ws_status.to_string(), COLOR_DIM),
    };
    f.render_widget(
        Paragraph::new(Span::styled(
            &status_label,
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_DIM))
                .title(Span::styled(" Status ", Style::default().fg(COLOR_DIM))),
        )
        .style(Style::default().bg(COLOR_PANEL)),
        chunks[1],
    );

    // Messages log
    let is_msgs_focused = app.focused == FocusedPanel::WsMessages;
    let msg_block = Block::default()
        .borders(Borders::ALL)
        .border_style(focused_border_color(is_msgs_focused, COLOR_WS))
        .title(Span::styled(" Messages ", Style::default().fg(COLOR_WS)))
        .style(Style::default().bg(COLOR_PANEL));

    let msg_lines: Vec<Line> = if app.ws_messages.is_empty() {
        vec![Line::from(Span::styled(
            "No messages yet — press ENTER to connect",
            Style::default().fg(COLOR_DIM),
        ))]
    } else {
        app.ws_messages
            .iter()
            .map(|m| Line::from(Span::styled(m, Style::default().fg(COLOR_FG))))
            .collect()
    };

    f.render_widget(
        Paragraph::new(Text::from(msg_lines))
            .block(msg_block)
            .wrap(Wrap { trim: false }),
        chunks[2],
    );

    // Input
    draw_editable_panel(
        f,
        chunks[3],
        " Send Message (ENTER) ",
        &app.ws_input,
        app.focused == FocusedPanel::WsInput,
        COLOR_WS,
        app.cursor_pos,
    );
}

// ─── Status bar ───────────────────────────────────────────────────────────────

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let protocol_color = match app.protocol {
        ProtocolMode::Http => COLOR_ACCENT,
        ProtocolMode::GraphQL => COLOR_GQL,
        ProtocolMode::WebSocket => COLOR_WS,
    };

    let mut hints = vec![("F6", "next panel"), ("F7", "prev panel")];

    let is_editable = matches!(
        app.focused,
        FocusedPanel::Url
            | FocusedPanel::Headers
            | FocusedPanel::Body
            | FocusedPanel::GqlEndpoint
            | FocusedPanel::GqlQuery
            | FocusedPanel::GqlVariables
            | FocusedPanel::WsUrl
            | FocusedPanel::WsInput
    );

    if is_editable {
        hints.push(("PgUp/Dn", "history"));
    }

    match app.protocol {
        ProtocolMode::Http => {
            hints.push(("Ctrl+S", "send"));
            if app.focused == FocusedPanel::Response {
                hints.push(("y/c", "copy"));
                hints.push(("↑/↓", "scroll"));
                hints.push(("ENTER", "resend"));
            } else {
                hints.push(("SPACE", "cycle method"));
                hints.push(("ENTER", "send"));
            }
        }
        ProtocolMode::GraphQL => {
            hints.push(("Ctrl+S", "send"));
            if app.focused == FocusedPanel::GqlResponse {
                hints.push(("y/c", "copy"));
                hints.push(("↑/↓", "scroll"));
                hints.push(("ENTER", "resend"));
            } else {
                hints.push(("ENTER", "send"));
                hints.push(("Ctrl+Q", "query"));
                hints.push(("Ctrl+V", "vars"));
            }
        }
        ProtocolMode::WebSocket => {
            hints.push(("ENTER", "send/connect"));
        }
    }

    hints.push(("Ctrl+P", "protocol"));
    hints.push(("Ctrl+C", "quit"));

    let mut spans: Vec<Span> = vec![];
    for (key, desc) in hints {
        spans.push(Span::styled(
            format!(" {key} "),
            Style::default()
                .fg(COLOR_BG)
                .bg(protocol_color)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {desc}  "),
            Style::default().fg(COLOR_DIM),
        ));
    }

    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(COLOR_BG)),
        area,
    );
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn focused_border(active: bool) -> Style {
    if active {
        Style::default()
            .fg(COLOR_ACCENT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(COLOR_DIM)
    }
}

fn focused_border_color(active: bool, color: Color) -> Style {
    if active {
        Style::default().fg(color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(COLOR_DIM)
    }
}

fn with_cursor(text: &str, cursor_pos: usize) -> String {
    let mut s = String::new();
    let mut chars = text.chars();
    for _ in 0..cursor_pos {
        if let Some(c) = chars.next() {
            s.push(c);
        }
    }
    s.push('█');
    for c in chars {
        s.push(c);
    }
    s
}

fn calculate_scroll(text: &str, cursor_pos: usize, area_height: u16) -> u16 {
    let up_to_cursor: String = text.chars().take(cursor_pos).collect();
    let current_line = up_to_cursor.split('\n').count().saturating_sub(1) as u16;
    let inner_height = area_height.saturating_sub(2);
    if inner_height == 0 {
        return 0;
    }
    if current_line >= inner_height {
        current_line - inner_height + 1
    } else {
        0
    }
}

fn draw_editable_panel(
    f: &mut Frame,
    area: Rect,
    title: &str,
    content: &str,
    is_focused: bool,
    color: Color,
    cursor_pos: usize,
) {
    let display = if is_focused {
        with_cursor(content, cursor_pos)
    } else {
        content.to_string()
    };

    let scroll_y = if is_focused {
        calculate_scroll(content, cursor_pos, area.height)
    } else {
        0
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(focused_border_color(is_focused, color))
        .title(Span::styled(title, Style::default().fg(color)))
        .style(Style::default().bg(COLOR_PANEL));

    f.render_widget(
        Paragraph::new(display)
            .block(block)
            .style(Style::default().fg(COLOR_FG))
            .wrap(Wrap { trim: false })
            .scroll((scroll_y, 0)),
        area,
    );
}

fn simple_panel(
    f: &mut Frame,
    area: Rect,
    title: &str,
    message: &str,
    msg_color: Color,
    is_focused: bool,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(focused_border(is_focused))
        .title(Span::styled(title, Style::default().fg(COLOR_DIM)))
        .style(Style::default().bg(COLOR_PANEL));
    f.render_widget(
        Paragraph::new(message)
            .block(block)
            .style(Style::default().fg(msg_color)),
        area,
    );
}

fn error_block(title: &str) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_ERROR))
        .title(Span::styled(
            title.to_string(),
            Style::default().fg(COLOR_ERROR),
        ))
        .style(Style::default().bg(COLOR_PANEL))
}
