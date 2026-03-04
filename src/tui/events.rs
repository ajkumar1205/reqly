use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use super::app::{App, FocusedPanel, ProtocolMode};

/// The result of processing an event.
pub enum EventOutcome {
    Continue,
    SendRequest,
    Quit,
}

/// Poll for terminal events (50ms timeout) and apply them to `app`.
pub fn handle_events(app: &mut App) -> Result<EventOutcome> {
    if !event::poll(Duration::from_millis(50))? {
        return Ok(EventOutcome::Continue);
    }
    if let Event::Key(key) = event::read()? {
        return Ok(process_key(app, key));
    }
    Ok(EventOutcome::Continue)
}

fn process_key(app: &mut App, key: KeyEvent) -> EventOutcome {
    // ── Global shortcuts ──────────────────────────────────────────────────────
    match (key.modifiers, key.code) {
        // Quit
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => return EventOutcome::Quit,

        // Cycle protocol
        (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
            app.cycle_protocol();
            return EventOutcome::Continue;
        }

        // Tab / Shift-Tab — move between panels within the active protocol
        (KeyModifiers::NONE, KeyCode::Tab) => {
            app.focus_next();
            return EventOutcome::Continue;
        }
        (KeyModifiers::SHIFT, KeyCode::BackTab) => {
            app.focus_prev();
            return EventOutcome::Continue;
        }

        // HTTP-specific jump shortcuts
        (KeyModifiers::CONTROL, KeyCode::Char('u')) if app.protocol == ProtocolMode::Http => {
            app.focused = FocusedPanel::Url;
            app.cursor_pos = app.url.len();
            return EventOutcome::Continue;
        }
        (KeyModifiers::CONTROL, KeyCode::Char('h')) if app.protocol == ProtocolMode::Http => {
            app.focused = FocusedPanel::Headers;
            app.cursor_pos = app.headers_raw.len();
            return EventOutcome::Continue;
        }
        (KeyModifiers::CONTROL, KeyCode::Char('b')) if app.protocol == ProtocolMode::Http => {
            app.focused = FocusedPanel::Body;
            app.cursor_pos = app.body_raw.len();
            return EventOutcome::Continue;
        }

        // GraphQL-specific jump shortcuts
        (KeyModifiers::CONTROL, KeyCode::Char('q')) if app.protocol == ProtocolMode::GraphQL => {
            app.focused = FocusedPanel::GqlQuery;
            app.cursor_pos = app.gql_query.len();
            return EventOutcome::Continue;
        }
        (KeyModifiers::CONTROL, KeyCode::Char('v')) if app.protocol == ProtocolMode::GraphQL => {
            app.focused = FocusedPanel::GqlVariables;
            app.cursor_pos = app.gql_variables.len();
            return EventOutcome::Continue;
        }

        // Ctrl+S — global force send request
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
            return EventOutcome::SendRequest;
        }

        // ENTER — behavior based on focused panel
        (KeyModifiers::NONE, KeyCode::Enter) => {
            let triggers_send = matches!(
                app.focused,
                // Single-line inputs trigger send
                FocusedPanel::Method
                | FocusedPanel::Url
                | FocusedPanel::GqlEndpoint
                | FocusedPanel::WsUrl
                | FocusedPanel::WsInput
                // Response panels resend
                | FocusedPanel::Response
                | FocusedPanel::GqlResponse
            );

            if triggers_send {
                return EventOutcome::SendRequest;
            }

            // If it's a multiline editable panel (Headers, Body, GqlQuery, GqlVariables),
            // we do NOT return here — it falls through to the editable panel section
            // below to properly insert a '\n' character!
        }

        // SPACE on Method panel — cycle HTTP method
        (KeyModifiers::NONE, KeyCode::Char(' ')) if app.focused == FocusedPanel::Method => {
            app.cycle_method();
            return EventOutcome::Continue;
        }

        // 'y' or 'c' on Response panel to yank body
        (KeyModifiers::NONE, KeyCode::Char('y')) | (KeyModifiers::NONE, KeyCode::Char('c')) => {
            if app.focused == FocusedPanel::Response {
                if let Some(Ok(resp)) = &app.response {
                    let mut clipboard = arboard::Clipboard::new().unwrap();
                    let _ = clipboard.set_text(resp.body.clone());
                }
                return EventOutcome::Continue;
            } else if app.focused == FocusedPanel::GqlResponse {
                if let Some(Ok(resp)) = &app.gql_response {
                    let mut clipboard = arboard::Clipboard::new().unwrap();
                    let _ = clipboard.set_text(resp.pretty_body());
                }
                return EventOutcome::Continue;
            }
        }

        // Up/Down scrolling for Response panes OR navigation for editable panes
        (KeyModifiers::NONE, KeyCode::Up) => {
            if app.focused == FocusedPanel::Response {
                app.response_scroll = app.response_scroll.saturating_sub(1);
                return EventOutcome::Continue;
            } else if app.focused == FocusedPanel::GqlResponse {
                app.gql_response_scroll = app.gql_response_scroll.saturating_sub(1);
                return EventOutcome::Continue;
            } else if app.focused == FocusedPanel::WsMessages {
                // ws scrolling not implemented yet, just return
                return EventOutcome::Continue;
            }
        }
        (KeyModifiers::NONE, KeyCode::Down) => {
            if app.focused == FocusedPanel::Response {
                app.response_scroll = app.response_scroll.saturating_add(1);
                return EventOutcome::Continue;
            } else if app.focused == FocusedPanel::GqlResponse {
                app.gql_response_scroll = app.gql_response_scroll.saturating_add(1);
                return EventOutcome::Continue;
            } else if app.focused == FocusedPanel::WsMessages {
                // ws scrolling not implemented yet
                return EventOutcome::Continue;
            }
        }

        _ => {}
    }

    // ── Editable panels ───────────────────────────────────────────────────────
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
        match key.code {
            KeyCode::Char(c) => app.insert_char(c),
            KeyCode::Backspace => app.delete_char(),
            KeyCode::Left => app.move_cursor_left(),
            KeyCode::Right => app.move_cursor_right(),
            KeyCode::Up => app.move_cursor_up(),
            KeyCode::Down => app.move_cursor_down(),
            KeyCode::Enter => {
                // Newlines allowed in multiline panels (headers, body, gql query, gql vars)
                let multiline = matches!(
                    app.focused,
                    FocusedPanel::Headers
                        | FocusedPanel::Body
                        | FocusedPanel::GqlQuery
                        | FocusedPanel::GqlVariables
                );
                if multiline {
                    app.insert_char('\n');
                }
            }
            _ => {}
        }
    }

    EventOutcome::Continue
}
