use crate::app::{App, Mode};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub enum Action {
    None,
    SendMessage(String),
    ExecuteCommand(String),
    Quit,
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> Action {
    match app.mode {
        Mode::Normal => handle_normal(app, key),
        Mode::Insert => handle_insert(app, key),
        Mode::Command => handle_command(app, key),
    }
}

fn handle_normal(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('i') => {
            app.mode = Mode::Insert;
            Action::None
        }
        KeyCode::Char(':') => {
            app.mode = Mode::Command;
            Action::None
        }
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('j') | KeyCode::Down => {
            app.scroll_offset = app.scroll_offset.saturating_add(1);
            Action::None
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.scroll_offset = app.scroll_offset.saturating_sub(1);
            Action::None
        }
        KeyCode::Char('G') => {
            app.scroll_offset = u16::MAX;
            Action::None
        }
        KeyCode::Char('g') => {
            app.scroll_offset = 0;
            Action::None
        }
        _ => Action::None,
    }
}

fn handle_insert(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            Action::None
        }
        KeyCode::Enter => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                app.insert_char('\n');
                Action::None
            } else {
                let text = app.take_input();
                if text.trim().is_empty() {
                    Action::None
                } else {
                    Action::SendMessage(text)
                }
            }
        }
        KeyCode::Backspace => {
            app.backspace();
            Action::None
        }
        KeyCode::Char(ch) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) && ch == 'd' {
                return Action::Quit;
            }
            app.insert_char(ch);
            Action::None
        }
        _ => Action::None,
    }
}

fn handle_command(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => {
            app.command_input.clear();
            app.command_cursor = 0;
            app.mode = Mode::Normal;
            Action::None
        }
        KeyCode::Enter => {
            let cmd = app.take_command();
            if cmd.trim().is_empty() {
                return Action::None;
            }
            Action::ExecuteCommand(cmd)
        }
        KeyCode::Backspace => {
            app.backspace();
            if app.command_input.is_empty() {
                app.mode = Mode::Normal;
            }
            Action::None
        }
        KeyCode::Char(ch) => {
            app.insert_char(ch);
            Action::None
        }
        _ => Action::None,
    }
}
