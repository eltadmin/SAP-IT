//! Event handling for the TUI.

use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

/// Application events.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Event {
    /// Terminal tick (for animations/updates).
    Tick,
    /// Keyboard event.
    Key(KeyEvent),
    /// Mouse event (unused but available).
    Mouse(event::MouseEvent),
    /// Terminal resize.
    Resize(u16, u16),
}

/// Event handler that polls for terminal events.
pub struct EventHandler {
    /// Tick rate for updates.
    tick_rate: Duration,
}

impl EventHandler {
    /// Create a new event handler with the given tick rate.
    pub fn new(tick_rate_ms: u64) -> Self {
        Self {
            tick_rate: Duration::from_millis(tick_rate_ms),
        }
    }

    /// Poll for the next event.
    pub fn next(&self) -> anyhow::Result<Event> {
        if event::poll(self.tick_rate)? {
            match event::read()? {
                CrosstermEvent::Key(key) => Ok(Event::Key(key)),
                CrosstermEvent::Mouse(mouse) => Ok(Event::Mouse(mouse)),
                CrosstermEvent::Resize(w, h) => Ok(Event::Resize(w, h)),
                _ => Ok(Event::Tick),
            }
        } else {
            Ok(Event::Tick)
        }
    }
}

/// Handle keyboard input for the application.
pub fn handle_key_event(app: &mut super::app::App, key: KeyEvent) {
    use super::app::Screen;

    // Global shortcuts
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.request_quit();
            return;
        }
        KeyCode::Char('q') if app.screen != Screen::EditServer => {
            app.request_quit();
            return;
        }
        _ => {}
    }

    // Screen-specific handling
    match app.screen {
        Screen::ServerList => handle_server_list(app, key),
        Screen::ConnectionTypeSelect => handle_connection_type(app, key),
        Screen::Connecting => handle_connecting(app, key),
        Screen::Connected => handle_connected(app, key),
        Screen::Help => handle_help(app, key),
        Screen::Settings => handle_settings(app, key),
        Screen::EditServer => handle_edit_server(app, key),
        Screen::Confirm => handle_confirm(app, key),
    }
}

fn handle_server_list(app: &mut super::app::App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Enter | KeyCode::Char(' ') => app.confirm_selection(),
        KeyCode::Char('a') => app.add_server(),
        KeyCode::Char('e') => app.edit_selected_server(),
        KeyCode::Char('d') | KeyCode::Delete => app.delete_selected_server(),
        KeyCode::Char('?') | KeyCode::F(1) => app.go_to_screen(super::app::Screen::Help),
        KeyCode::Char('s') => app.go_to_screen(super::app::Screen::Settings),
        KeyCode::Char('r') => {
            // Quick RDP connect
            if app.current_server().is_some() {
                app.selected_conn_type = 0;
                app.confirm_selection();
            }
        }
        KeyCode::Char('S') => {
            // Quick SSH connect (if available)
            if app.current_server().map(|s| s.has_ssh()).unwrap_or(false) {
                app.selected_conn_type = 1;
                app.confirm_selection();
                app.confirm_selection();
            }
        }
        KeyCode::Char('1'..='9') => {
            let index = key.code.to_string().parse::<usize>().unwrap_or(1) - 1;
            if index < app.config.servers.len() {
                app.selected_server = index;
                app.confirm_selection();
            }
        }
        _ => {}
    }
}

fn handle_connection_type(app: &mut super::app::App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Enter | KeyCode::Char(' ') => app.confirm_selection(),
        KeyCode::Esc | KeyCode::Backspace => app.go_back(),
        KeyCode::Char('1') => {
            app.selected_conn_type = 0;
            app.confirm_selection();
        }
        KeyCode::Char('2') => {
            let types = app.available_connection_types();
            if types.len() > 1 {
                app.selected_conn_type = 1;
                app.confirm_selection();
            }
        }
        KeyCode::Char('3') => {
            let types = app.available_connection_types();
            if types.len() > 2 {
                app.selected_conn_type = 2;
                app.confirm_selection();
            }
        }
        _ => {}
    }
}

fn handle_connecting(app: &mut super::app::App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.disconnect();
        }
        _ => {}
    }
}

fn handle_connected(app: &mut super::app::App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Enter => app.confirm_selection(),
        KeyCode::Char('d') => {
            app.confirm_action = Some(super::app::ConfirmAction::Disconnect);
            app.confirm_selection = 0;
            app.go_to_screen(super::app::Screen::Confirm);
        }
        _ => {}
    }
}

fn handle_help(app: &mut super::app::App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('?') | KeyCode::F(1) => app.go_back(),
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::PageUp => {
            for _ in 0..10 {
                app.select_previous();
            }
        }
        KeyCode::PageDown => {
            for _ in 0..10 {
                app.select_next();
            }
        }
        _ => {}
    }
}

fn handle_settings(app: &mut super::app::App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('s') => app.go_back(),
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Char('S') => {
            // Save config
            if let Err(e) = app.save_config() {
                app.log_status(format!("Failed to save config: {}", e));
            } else {
                app.log_status("Configuration saved");
            }
        }
        _ => {}
    }
}

fn handle_edit_server(app: &mut super::app::App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.go_back();
        }
        KeyCode::Enter => {
            app.confirm_selection();
        }
        KeyCode::Tab | KeyCode::Down => {
            app.save_current_field();
            app.select_next();
        }
        KeyCode::BackTab | KeyCode::Up => {
            app.save_current_field();
            app.select_previous();
        }
        KeyCode::Backspace => {
            app.handle_backspace();
        }
        KeyCode::Delete => {
            app.handle_delete();
        }
        KeyCode::Left => {
            app.cursor_left();
        }
        KeyCode::Right => {
            app.cursor_right();
        }
        KeyCode::Home => {
            app.cursor_position = 0;
        }
        KeyCode::End => {
            app.cursor_position = app.input_buffer.len();
        }
        KeyCode::Char(c) => {
            app.handle_char(c);
        }
        _ => {}
    }
}

fn handle_confirm(app: &mut super::app::App, key: KeyEvent) {
    match key.code {
        KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
            app.confirm_selection = if app.confirm_selection == 0 { 1 } else { 0 };
        }
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.confirm_selection = 1;
            app.confirm_selection();
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.confirm_selection = 0;
            app.confirm_action = None;
            app.go_back();
        }
        KeyCode::Enter => {
            app.confirm_selection();
        }
        _ => {}
    }
}
