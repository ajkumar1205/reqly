use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::app::{App, FocusedPanel};
use crate::formatter;

// Colour palette
const COLOR_ACCENT: Color = Color::Rgb(80, 200, 200); // teal accent
const COLOR_DIM: Color = Color::Rgb(100, 100, 120);
const COLOR_FG: Color = Color::Rgb(220, 220, 235);
const COLOR_BG: Color = Color::Rgb(18, 18, 28);
const COLOR_PANEL: Color = Color::Rgb(28, 28, 42);
const COLOR_SUCCESS: Color = Color::Rgb(80, 200, 120);
const COLOR_ERROR: Color = Color::Rgb(220, 80, 80);
const COLOR_METHOD: Color = Color::Rgb(240, 180, 50);

/// Render the full TUI.
pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    // Background fill
    f.render_widget(Block::default().style(Style::default().bg(COLOR_BG)), size);

    // Main vertical split: top (request) | bottom (response)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title bar
            Constraint::Length(3), // method + url bar
            Constraint::Length(8), // headers
            Constraint::Min(6),    // body
            Constraint::Min(10),   // response
            Constraint::Length(1), // status bar
        ])
        .split(size);

    draw_title_bar(f, main_chunks[0]);
    draw_method_url(f, app, main_chunks[1]);
    draw_headers(f, app, main_chunks[2]);
    draw_body(f, app, main_chunks[3]);
    draw_response(f, app, main_chunks[4]);
    draw_status_bar(f, app, main_chunks[5]);
}

fn draw_title_bar(f: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " ⚡ reqly ",
            Style::default()
                .fg(COLOR_ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("— terminal API client", Style::default().fg(COLOR_DIM)),
    ]))
    .style(Style::default().bg(COLOR_BG));
    f.render_widget(title, area);
}

fn draw_method_url(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(10), Constraint::Min(20)])
        .split(area);

    // Method button
    let method_style = if app.focused == FocusedPanel::Method {
        Style::default()
            .fg(COLOR_METHOD)
            .bg(COLOR_PANEL)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(COLOR_METHOD).bg(COLOR_PANEL)
    };
    let method_block = Block::default()
        .borders(Borders::ALL)
        .border_style(focused_border_style(app.focused == FocusedPanel::Method))
        .title(Span::styled(" Method ", Style::default().fg(COLOR_DIM)));
    let method_para = Paragraph::new(app.method.as_str())
        .block(method_block)
        .style(method_style);
    f.render_widget(method_para, chunks[0]);

    // URL field
    let url_block = Block::default()
        .borders(Borders::ALL)
        .border_style(focused_border_style(app.focused == FocusedPanel::Url))
        .title(Span::styled(" URL ", Style::default().fg(COLOR_DIM)));
    let url_text = if app.focused == FocusedPanel::Url {
        format!("{}_", app.url) // simple cursor indicator
    } else {
        app.url.clone()
    };
    let url_para = Paragraph::new(url_text)
        .block(url_block)
        .style(Style::default().fg(COLOR_FG).bg(COLOR_PANEL));
    f.render_widget(url_para, chunks[1]);
}

fn draw_headers(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(focused_border_style(app.focused == FocusedPanel::Headers))
        .title(Span::styled(
            " Headers  (one per line: Key: Value) ",
            Style::default().fg(COLOR_DIM),
        ))
        .style(Style::default().bg(COLOR_PANEL));

    let content = if app.focused == FocusedPanel::Headers {
        format!("{}_", app.headers_raw)
    } else {
        app.headers_raw.clone()
    };

    let para = Paragraph::new(content)
        .block(block)
        .style(Style::default().fg(COLOR_FG))
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn draw_body(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(focused_border_style(app.focused == FocusedPanel::Body))
        .title(Span::styled(
            " Body (JSON) ",
            Style::default().fg(COLOR_DIM),
        ))
        .style(Style::default().bg(COLOR_PANEL));

    let content = if app.focused == FocusedPanel::Body {
        format!("{}_", app.body_raw)
    } else {
        app.body_raw.clone()
    };

    let para = Paragraph::new(content)
        .block(block)
        .style(Style::default().fg(COLOR_FG))
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn draw_response(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused == FocusedPanel::Response;

    match &app.response {
        None if app.is_loading => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(focused_border_style(is_focused))
                .title(Span::styled(" Response ", Style::default().fg(COLOR_DIM)))
                .style(Style::default().bg(COLOR_PANEL));
            let para = Paragraph::new("⏳ Sending request…")
                .block(block)
                .style(Style::default().fg(COLOR_DIM));
            f.render_widget(para, area);
        }
        None => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(focused_border_style(is_focused))
                .title(Span::styled(" Response ", Style::default().fg(COLOR_DIM)))
                .style(Style::default().bg(COLOR_PANEL));
            let para = Paragraph::new("Press  ENTER  to send the request")
                .block(block)
                .style(Style::default().fg(COLOR_DIM));
            f.render_widget(para, area);
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
            let para = Paragraph::new(e.as_str())
                .block(block)
                .style(Style::default().fg(COLOR_ERROR))
                .wrap(Wrap { trim: false });
            f.render_widget(para, area);
        }
        Some(Ok(resp)) => {
            let status_color = if resp.status < 300 {
                COLOR_SUCCESS
            } else {
                COLOR_ERROR
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(focused_border_style(is_focused))
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

            // Collect up to 3 important headers
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

            let para = Paragraph::new(Text::from(lines))
                .block(block)
                .wrap(Wrap { trim: false });
            f.render_widget(para, area);
        }
    }
}

fn draw_status_bar(f: &mut Frame, _app: &App, area: Rect) {
    let hints = vec![
        ("TAB", "next panel"),
        ("SHIFT+TAB", "prev panel"),
        ("ENTER", "send"),
        ("SPACE", "cycle method"),
        ("Ctrl+C", "quit"),
    ];
    let mut spans: Vec<Span> = vec![];
    for (key, desc) in &hints {
        spans.push(Span::styled(
            format!(" {key} "),
            Style::default()
                .fg(COLOR_BG)
                .bg(COLOR_ACCENT)
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

fn focused_border_style(is_focused: bool) -> Style {
    if is_focused {
        Style::default()
            .fg(COLOR_ACCENT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(COLOR_DIM)
    }
}
