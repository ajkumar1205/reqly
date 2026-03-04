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

        // ENTER — send / connect
        (KeyModifiers::NONE, KeyCode::Enter) => {
            let is_response_panel = matches!(
                app.focused,
                FocusedPanel::Response | FocusedPanel::GqlResponse | FocusedPanel::WsMessages
            );
            if !is_response_panel {
                return EventOutcome::SendRequest;
            }
        }

        // SPACE on Method panel — cycle HTTP method
        (KeyModifiers::NONE, KeyCode::Char(' ')) if app.focused == FocusedPanel::Method => {
            app.cycle_method();
            return EventOutcome::Continue;
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
