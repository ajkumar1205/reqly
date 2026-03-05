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
pub fn handle_events(
    app: &mut App,
    db_conn: &mut Option<rusqlite::Connection>,
) -> Result<EventOutcome> {
    if !event::poll(Duration::from_millis(50))? {
        return Ok(EventOutcome::Continue);
    }
    if let Event::Key(key) = event::read()? {
        return Ok(process_key(app, key, db_conn));
    }
    Ok(EventOutcome::Continue)
}

fn process_key(
    app: &mut App,
    key: KeyEvent,
    db_conn: &mut Option<rusqlite::Connection>,
) -> EventOutcome {
    // ── Global shortcuts ──────────────────────────────────────────────────────
    match (key.modifiers, key.code) {
        // Quit
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => return EventOutcome::Quit,

        // Cycle protocol
        (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
            app.cycle_protocol();
            return EventOutcome::Continue;
        }

        // Ctrl+Tab / Shift+Tab — move between panels within the active protocol
        (KeyModifiers::CONTROL, KeyCode::Tab) => {
            app.focus_next();
            return EventOutcome::Continue;
        }
        (KeyModifiers::SHIFT, KeyCode::BackTab) => {
            app.focus_prev();
            return EventOutcome::Continue;
        }
        // If it's a single-line simple field, pure Tab also jumps next
        (KeyModifiers::NONE, KeyCode::Tab) if !app.is_multiline_focused() => {
            app.focus_next();
            return EventOutcome::Continue;
        }
        // Removed duplicate BackTab block

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

        // PageUp or Ctrl+Up to cycle older history
        (KeyModifiers::CONTROL, KeyCode::Up) => {
            // Re-use the same PageUp logic that will be defined below for editable panels
            // We just map it here to be safe if they use Ctrl+Up instead of PageUp.
            // Rather than duplicate, we can just translate the event, but for simplicity we'll let it drop through to the editable block
            // since this block returns EventOutcome. We will NOT return EventOutcome here so it drops to matched keys.
        }

        // PageDown or Ctrl+Down to cycle newer history
        (KeyModifiers::CONTROL, KeyCode::Down) => {
            // Same as above
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
        // Map Ctrl+Up to PageUp and Ctrl+Down to PageDown for history
        let canonical_code = match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Up) => KeyCode::PageUp,
            (KeyModifiers::CONTROL, KeyCode::Down) => KeyCode::PageDown,
            (_, code) => code,
        };

        match canonical_code {
            KeyCode::Char(c) => app.insert_char(c),
            KeyCode::Backspace => app.delete_char(),
            KeyCode::Left => app.move_cursor_left(),
            KeyCode::Right => app.move_cursor_right(),
            KeyCode::Up => app.move_cursor_up(),
            KeyCode::Down => app.move_cursor_down(),
            KeyCode::Tab => {
                if app.is_multiline_focused() {
                    app.insert_str("    ");
                }
            }
            KeyCode::PageUp => {
                if app.history_cache.is_none() {
                    let fetched = app.fetch_history_for_panel(db_conn);
                    app.history_cache = Some(fetched);
                }

                let history_len = app.history_cache.as_ref().unwrap().len();
                if history_len > 0 {
                    // Save draft on first navigation away from current
                    if app.history_index == 0 && app.working_draft.is_none() {
                        app.working_draft = Some(app.active_text().to_string());
                    }

                    if app.history_index < history_len {
                        app.history_index += 1;
                        let next_text =
                            app.history_cache.as_ref().unwrap()[app.history_index - 1].clone();

                        if let Some(buf) = app.active_text_mut() {
                            *buf = next_text;
                        }
                        app.cursor_pos = app.active_text().len();
                    }
                }
            }
            KeyCode::PageDown => {
                if app.history_index > 0 {
                    app.history_index -= 1;

                    if app.history_index == 0 {
                        // Restore draft
                        if let Some(draft) = app.working_draft.take() {
                            if let Some(buf) = app.active_text_mut() {
                                *buf = draft;
                            }
                        }
                    } else {
                        // Go newer in history
                        if app.history_cache.is_none() {
                            let fetched = app.fetch_history_for_panel(db_conn);
                            app.history_cache = Some(fetched);
                        }
                        let next_text =
                            app.history_cache.as_ref().unwrap()[app.history_index - 1].clone();
                        if let Some(buf) = app.active_text_mut() {
                            *buf = next_text;
                        }
                    }
                    app.cursor_pos = app.active_text().len();
                }
            }
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
