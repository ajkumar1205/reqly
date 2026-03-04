mod cli;
mod formatter;
mod http;
mod tui;
mod utils;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::CrosstermBackend, Terminal};
use std::io;

use cli::{Cli, ParsedRequest};
use http::{HttpClient, HttpRequest};
use tui::{
    app::App,
    events::{handle_events, EventOutcome},
    ui::draw,
};
use utils::parse_header;

// ─── Entry point ─────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        // ── CLI mode ──────────────────────────────────────────────────────────
        Some(command) => {
            let parsed = ParsedRequest::from_command(command);
            run_cli(parsed).await?;
        }

        // ── Interactive TUI mode ──────────────────────────────────────────────
        None => {
            run_tui().await?;
        }
    }

    Ok(())
}

// ─── CLI Mode ────────────────────────────────────────────────────────────────

async fn run_cli(parsed: ParsedRequest) -> Result<()> {
    // Build the request
    let mut req = HttpRequest::new(&parsed.method, &parsed.url);

    for raw in &parsed.headers {
        if let Some((k, v)) = parse_header(raw) {
            req.headers.insert(k, v);
        } else {
            eprintln!("{} Skipping malformed header: {}", "warn:".yellow(), raw);
        }
    }

    if let Some(body) = parsed.body {
        req.body = Some(body);
    }

    // Ensure Content-Type is set for requests with a body
    if req.body.is_some() && !req.headers.contains_key("content-type") {
        req.headers
            .insert("Content-Type".into(), "application/json".into());
    }

    println!();
    println!("{}", format!("→ {} {}", req.method, req.url).cyan().bold());
    println!();

    let client = HttpClient::new()?;
    let response = client.send(req).await.map_err(|e| {
        eprintln!("{} {}", "error:".red().bold(), e);
        e
    })?;

    // ── Status line ───────────────────────────────────────────────────────────
    let status_str = format!("{} {}", response.status, response.status_text);
    let status_colored = if response.status < 300 {
        status_str.green().bold()
    } else if response.status < 500 {
        status_str.yellow().bold()
    } else {
        status_str.red().bold()
    };

    println!("  {}  {}", "Status:".bold(), status_colored);
    println!(
        "  {}  {}ms",
        "Time:  ".bold(),
        response.duration_ms.to_string().cyan()
    );
    println!();

    // ── Response headers ──────────────────────────────────────────────────────
    println!("{}", "Headers".bold().underline());
    println!("{}", "─".repeat(40).dimmed());
    for (k, v) in &response.headers {
        println!("  {}: {}", k.dimmed(), v);
    }
    println!();

    // ── Response body ─────────────────────────────────────────────────────────
    println!("{}", "Body".bold().underline());
    println!("{}", "─".repeat(40).dimmed());

    let body = if response.is_json() {
        formatter::pretty_print(&response.body)
    } else {
        response.body.clone()
    };

    println!("{}", body);
    println!();

    Ok(())
}

// ─── TUI Mode ────────────────────────────────────────────────────────────────

async fn run_tui() -> Result<()> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let result = run_tui_loop(&mut terminal, &mut app).await;

    // Restore terminal regardless of outcome
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_tui_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        // Draw frame
        terminal.draw(|f| draw(f, app))?;

        // Handle keyboard events
        let outcome = handle_events(app)?;

        match outcome {
            EventOutcome::Quit => break,

            EventOutcome::SendRequest => {
                if app.url.trim().is_empty() {
                    continue;
                }
                app.is_loading = true;
                app.response = None;

                // Build request from TUI state
                let mut req = HttpRequest::new(app.method.clone(), app.url.trim());

                for line in app.headers_raw.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    if let Some((k, v)) = parse_header(line) {
                        req.headers.insert(k, v);
                    }
                }

                if !app.body_raw.trim().is_empty() {
                    req.body = Some(app.body_raw.trim().to_string());
                    if !req.headers.contains_key("content-type") {
                        req.headers
                            .insert("Content-Type".into(), "application/json".into());
                    }
                }

                // Send the request
                let client = HttpClient::new();
                match client {
                    Err(e) => {
                        app.response = Some(Err(e.to_string()));
                    }
                    Ok(client) => match client.send(req).await {
                        Ok(resp) => {
                            app.response = Some(Ok(resp));
                        }
                        Err(e) => {
                            app.response = Some(Err(e.to_string()));
                        }
                    },
                }

                app.is_loading = false;
                // Move focus to response panel
                app.focused = tui::app::FocusedPanel::Response;
            }

            EventOutcome::Continue => {}
        }
    }

    Ok(())
}
