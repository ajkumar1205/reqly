use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use super::app::{App, FocusedPanel};

/// The result of processing an event.
pub enum EventOutcome {
    /// Nothing special, keep running.
    Continue,
    /// User requested to send the request.
    SendRequest,
    /// User wants to quit.
    Quit,
}

/// Poll for terminal events with a timeout and apply them to `app`.
/// Returns the outcome so the main loop can react.
pub fn handle_events(app: &mut App) -> Result<EventOutcome> {
    // Poll with a short timeout so the UI stays responsive.
    if !event::poll(Duration::from_millis(50))? {
        return Ok(EventOutcome::Continue);
    }

    if let Event::Key(key) = event::read()? {
        return Ok(process_key(app, key));
    }

    Ok(EventOutcome::Continue)
}

fn process_key(app: &mut App, key: KeyEvent) -> EventOutcome {
    // Global shortcuts (work in any panel)
    match (key.modifiers, key.code) {
        // Quit
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => return EventOutcome::Quit,

        // Tab / Shift-Tab — move between panels
        (KeyModifiers::NONE, KeyCode::Tab) | (KeyModifiers::SHIFT, KeyCode::BackTab) => {
            if key.code == KeyCode::Tab {
                app.focus_next();
            } else {
                app.focus_prev();
            }
            return EventOutcome::Continue;
        }

        // CTRL shortcuts to jump directly to panels
        (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
            app.focused = FocusedPanel::Url;
            app.cursor_pos = app.url.len();
            return EventOutcome::Continue;
        }
        (KeyModifiers::CONTROL, KeyCode::Char('h')) => {
            app.focused = FocusedPanel::Headers;
            app.cursor_pos = app.headers_raw.len();
            return EventOutcome::Continue;
        }
        (KeyModifiers::CONTROL, KeyCode::Char('b')) => {
            app.focused = FocusedPanel::Body;
            app.cursor_pos = app.body_raw.len();
            return EventOutcome::Continue;
        }

        // Enter — send request (from any panel except Response)
        (KeyModifiers::NONE, KeyCode::Enter) if app.focused != FocusedPanel::Response => {
            return EventOutcome::SendRequest;
        }

        // Space on Method panel — cycle method
        (KeyModifiers::NONE, KeyCode::Char(' ')) if app.focused == FocusedPanel::Method => {
            app.cycle_method();
            return EventOutcome::Continue;
        }

        _ => {}
    }

    // Panel-specific editing
    match app.focused {
        FocusedPanel::Url | FocusedPanel::Headers | FocusedPanel::Body => {
            match key.code {
                KeyCode::Char(c) => app.insert_char(c),
                KeyCode::Backspace => app.delete_char(),
                KeyCode::Left => app.move_cursor_left(),
                KeyCode::Right => app.move_cursor_right(),
                KeyCode::Enter => {
                    // Allow newlines inside headers / body
                    if app.focused != FocusedPanel::Url {
                        app.insert_char('\n');
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }

    EventOutcome::Continue
}
