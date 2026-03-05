mod cli;
mod formatter;
mod protocols;
mod storage;
mod tui;
mod utils;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use std::collections::HashMap;
use std::io;

use cli::{Cli, Commands, ParsedRequest};
use protocols::graphql::{GraphqlClient, GraphqlQuery};
use protocols::http::{HttpClient, HttpRequest};
use protocols::websocket::WebSocketClient;
use protocols::websocket::message::WsMessage;
use tui::app::{App, FocusedPanel, ProtocolMode};
use tui::events::{EventOutcome, handle_events};
use tui::ui::draw;
use utils::parse_header;

// ─── Entry point ─────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        // GraphQL
        Some(Commands::Graphql {
            url,
            query,
            variables,
            operation,
            headers,
        }) => {
            run_cli_graphql(
                &url,
                &query,
                variables.as_deref(),
                operation.as_deref(),
                &headers,
            )
            .await?;
        }

        // WebSocket
        Some(Commands::Ws {
            url,
            headers,
            json,
            ping,
        }) => {
            run_cli_ws(&url, &headers, json, ping).await?;
        }

        // All HTTP methods
        Some(cmd) => {
            if let Some(parsed) = ParsedRequest::from_command(&cmd) {
                run_cli_http(parsed).await?;
            }
        }

        // TUI
        None => {
            // Init SQLite DB
            let mut db_conn = match storage::db::init_db() {
                Ok(c) => Some(c),
                Err(e) => {
                    eprintln!("Failed to initialize local request history DB: {}", e);
                    None
                }
            };

            let mut app = App::new();
            // Run TUI
            crossterm::terminal::enable_raw_mode()?;
            crossterm::execute!(io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
            let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

            let res = run_tui_loop(&mut terminal, &mut app, &mut db_conn).await;

            crossterm::terminal::disable_raw_mode()?;
            crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;

            if let Err(err) = res {
                println!("{:?}", err)
            }
        }
    }

    Ok(())
}

// ─── CLI dispatch is now handled inherently in main ───────────────────────────

// ─── HTTP CLI ─────────────────────────────────────────────────────────────────

async fn run_cli_http(parsed: ParsedRequest) -> Result<()> {
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

    println!("{}", "Headers".bold().underline());
    println!("{}", "─".repeat(40).dimmed());
    for (k, v) in &response.headers {
        println!("  {}: {}", k.dimmed(), v);
    }
    println!();

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

// ─── GraphQL CLI ──────────────────────────────────────────────────────────────

async fn run_cli_graphql(
    url: &str,
    query: &str,
    variables: Option<&str>,
    operation: Option<&str>,
    raw_headers: &[String],
) -> Result<()> {
    println!();
    println!("{}", format!("→ GraphQL  {url}").magenta().bold());
    println!();

    let mut gql_query = GraphqlQuery::new(url, query);

    // Parse and attach variables
    if let Some(vars_str) = variables {
        match serde_json::from_str(vars_str) {
            Ok(val) => {
                gql_query = gql_query.with_variables(val);
            }
            Err(_) => {
                eprintln!(
                    "{} Variables are not valid JSON — ignoring.",
                    "warn:".yellow()
                );
            }
        }
    }

    if let Some(op) = operation {
        gql_query = gql_query.with_operation(op);
    }

    for raw in raw_headers {
        if let Some((k, v)) = parse_header(raw) {
            gql_query = gql_query.with_header(k, v);
        }
    }

    let client = GraphqlClient::new()?;
    let response = client.execute(&gql_query).await.map_err(|e| {
        eprintln!("{} {}", "error:".red().bold(), e);
        e
    })?;

    // Status line
    let status_str = format!("{} {}", response.status, response.status_text);
    let status_colored = if response.status < 300 {
        status_str.green().bold()
    } else {
        status_str.red().bold()
    };
    println!("  {}  {}", "Status:".bold(), status_colored);
    println!(
        "  {}  {}ms",
        "Time:  ".bold(),
        response.duration_ms.to_string().cyan()
    );

    if response.has_errors() {
        println!("  {}", "⚠ GraphQL errors present".red().bold());
    }
    println!();

    // Body
    println!("{}", "Body".bold().underline());
    println!("{}", "─".repeat(40).dimmed());
    println!("{}", response.pretty_body());
    println!();

    Ok(())
}

// ─── WebSocket CLI ────────────────────────────────────────────────────────────

async fn run_cli_ws(
    url: &str,
    raw_headers: &[String],
    pretty_json: bool,
    _ping: bool,
) -> Result<()> {
    let mut headers = HashMap::new();
    for raw in raw_headers {
        if let Some((k, v)) = parse_header(raw) {
            headers.insert(k, v);
        }
    }

    WebSocketClient::run_interactive(url, &headers, pretty_json).await
}

// ─── TUI Mode ────────────────────────────────────────────────────────────────

// The run_tui wrapper was removed.

async fn run_tui_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    db_conn: &mut Option<rusqlite::Connection>,
) -> Result<()> {
    loop {
        terminal.draw(|f| draw(f, app))?;

        let outcome = handle_events(app, db_conn)?;

        match outcome {
            EventOutcome::Quit => break,

            EventOutcome::SendRequest => {
                // Set loading status and force exactly one render BEFORE blocking on network
                app.is_loading = true;
                terminal.draw(|f| draw(f, app))?;

                match app.protocol {
                    ProtocolMode::Http => tui_send_http(app, db_conn).await,
                    ProtocolMode::GraphQL => tui_send_graphql(app, db_conn).await,
                    ProtocolMode::WebSocket => tui_ws_connect_or_send(app, db_conn).await,
                }

                app.is_loading = false;
            }

            EventOutcome::Continue => {}
        }
    }

    Ok(())
}

// ─── TUI HTTP send ────────────────────────────────────────────────────────────

async fn tui_send_http(app: &mut App, db_conn: &mut Option<rusqlite::Connection>) {
    if app.url.trim().is_empty() {
        return;
    }
    app.response = None;

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

    match HttpClient::new() {
        Err(e) => app.response = Some(Err(e.to_string())),
        Ok(client) => match client.send(req).await {
            Ok(resp) => {
                app.focused = FocusedPanel::Response;
                let resp_body = resp.body.clone();
                let status_code = resp.status;
                let duration_ms = resp.duration_ms;

                if let Some(conn) = db_conn {
                    let _ = storage::queries::insert_request(
                        conn,
                        "HTTP",
                        app.url.trim(),
                        &app.method,
                        &app.headers_raw,
                        &app.body_raw,
                        &resp_body,
                        status_code as i64,
                        duration_ms as i64,
                    );
                }
                app.response = Some(Ok(resp));
            }
            Err(e) => {
                app.response = Some(Err(e.to_string()));
            }
        },
    }

    app.is_loading = false;
}

// ─── TUI GraphQL send ─────────────────────────────────────────────────────────

async fn tui_send_graphql(app: &mut App, _db_conn: &mut Option<rusqlite::Connection>) {
    if app.gql_endpoint.trim().is_empty() || app.gql_query.trim().is_empty() {
        return;
    }
    app.gql_response = None;

    let mut gql_query = GraphqlQuery::new(app.gql_endpoint.trim(), app.gql_query.trim());

    if !app.gql_variables.trim().is_empty() {
        if let Ok(val) = serde_json::from_str(&app.gql_variables) {
            gql_query = gql_query.with_variables(val);
        }
    }

    match GraphqlClient::new() {
        Err(e) => app.gql_response = Some(Err(e.to_string())),
        Ok(client) => match client.execute(&gql_query).await {
            Ok(resp) => {
                app.focused = FocusedPanel::GqlResponse;
                app.gql_response = Some(Ok(resp));
            }
            Err(e) => app.gql_response = Some(Err(e.to_string())),
        },
    }

    app.is_loading = false;
}

// ─── TUI WebSocket (informational — full interactive WS in TUI is CLI-focused)

async fn tui_ws_connect_or_send(app: &mut App, _db_conn: &mut Option<rusqlite::Connection>) {
    use protocols::websocket::ConnectionStatus;

    if app.ws_url.trim().is_empty() {
        app.push_ws_message("⚠ Enter a WebSocket URL first (ws:// or wss://)".to_string());
        return;
    }

    // If there's a message in the input box, add it to the log (TUI WS is display-only;
    // interactive mode is available via `reqly ws <url>` in CLI)
    if !app.ws_input.trim().is_empty() {
        let msg = format!("→ Sent: {}", app.ws_input.trim());
        app.push_ws_message(msg);
        app.ws_input.clear();
        app.cursor_pos = 0;
    } else {
        app.ws_status = ConnectionStatus::Connecting;
        app.push_ws_message(format!(
            "ℹ  Use  reqly ws {}  in the terminal for live WebSocket sessions.",
            app.ws_url.trim()
        ));
        app.ws_status = ConnectionStatus::Disconnected;
    }
}
