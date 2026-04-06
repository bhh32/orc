use crate::app::{App, AppView, EditFocus, Mode};
use crate::panes::editor::EditorMode;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use helix_core::movement::Direction;

pub enum Action {
    None,
    SendMessage(String),
    ExecuteCommand(String),
    Quit,
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> Action {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('e') => {
                app.toggle_view();
                return Action::None;
            }
            KeyCode::Char('d') => return Action::Quit,
            _ => {}
        }
    }

    match app.view {
        AppView::Chat => handle_chat_key(app, key),
        AppView::Edit => handle_edit_key(app, key),
    }
}

fn handle_chat_key(app: &mut App, key: KeyEvent) -> Action {
    match app.mode {
        Mode::Normal => handle_chat_normal(app, key),
        Mode::Insert => handle_chat_insert(app, key),
        Mode::Command => handle_command(app, key),
    }
}

fn handle_edit_key(app: &mut App, key: KeyEvent) -> Action {
    if app.mode == Mode::Command {
        return handle_command(app, key);
    }

    match app.edit_focus {
        EditFocus::Sidebar => handle_sidebar_key(app, key),
        EditFocus::Editor => handle_editor_key(app, key),
    }
}

fn handle_chat_normal(app: &mut App, key: KeyEvent) -> Action {
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

fn handle_chat_insert(app: &mut App, key: KeyEvent) -> Action {
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
        KeyCode::Up | KeyCode::PageUp => {
            app.scroll_offset = app.scroll_offset.saturating_sub(1);
            Action::None
        }
        KeyCode::Down | KeyCode::PageDown => {
            app.scroll_offset = app.scroll_offset.saturating_add(1);
            Action::None
        }
        KeyCode::Left => {
            app.input_cursor = app.input_cursor.saturating_sub(1);
            Action::None
        }
        KeyCode::Right => {
            if app.input_cursor < app.input.len() {
                app.input_cursor += 1;
            }
            Action::None
        }
        KeyCode::Home => {
            app.input_cursor = 0;
            Action::None
        }
        KeyCode::End => {
            app.input_cursor = app.input.len();
            Action::None
        }
        KeyCode::Char(ch) => {
            app.insert_char(ch);
            Action::None
        }
        _ => Action::None,
    }
}

fn handle_editor_key(app: &mut App, key: KeyEvent) -> Action {
    let buf = &mut app.buffer;

    match buf.mode {
        EditorMode::Normal => match key.code {
            KeyCode::Char('i') => {
                buf.mode = EditorMode::Insert;
                Action::None
            }
            KeyCode::Char('a') => {
                buf.move_cursor(Direction::Forward, 1);
                buf.mode = EditorMode::Insert;
                Action::None
            }
            KeyCode::Char('v') => {
                buf.mode = EditorMode::Select;
                Action::None
            }
            KeyCode::Char('h') | KeyCode::Left => {
                buf.move_cursor(Direction::Backward, 1);
                Action::None
            }
            KeyCode::Char('l') | KeyCode::Right => {
                buf.move_cursor(Direction::Forward, 1);
                Action::None
            }
            KeyCode::Char('j') | KeyCode::Down => {
                buf.move_line(Direction::Forward, 1);
                Action::None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                buf.move_line(Direction::Backward, 1);
                Action::None
            }
            KeyCode::Char('d') => {
                buf.delete_selection();
                Action::None
            }
            KeyCode::Char(':') => {
                app.mode = Mode::Command;
                Action::None
            }
            KeyCode::Tab => {
                app.edit_focus = EditFocus::Sidebar;
                Action::None
            }
            _ => Action::None,
        },
        EditorMode::Insert => match key.code {
            KeyCode::Esc => {
                buf.mode = EditorMode::Normal;
                Action::None
            }
            KeyCode::Enter => {
                buf.insert_newline();
                Action::None
            }
            KeyCode::Backspace => {
                buf.delete_backward();
                Action::None
            }
            KeyCode::Left => {
                buf.move_cursor(Direction::Backward, 1);
                Action::None
            }
            KeyCode::Right => {
                buf.move_cursor(Direction::Forward, 1);
                Action::None
            }
            KeyCode::Up => {
                buf.move_line(Direction::Backward, 1);
                Action::None
            }
            KeyCode::Down => {
                buf.move_line(Direction::Forward, 1);
                Action::None
            }
            KeyCode::Char(ch) => {
                buf.insert_char(ch);
                Action::None
            }
            _ => Action::None,
        },
        EditorMode::Select => match key.code {
            KeyCode::Esc => {
                buf.mode = EditorMode::Normal;
                Action::None
            }
            KeyCode::Char('d') => {
                buf.delete_selection();
                buf.mode = EditorMode::Normal;
                Action::None
            }
            KeyCode::Char('h') | KeyCode::Left => {
                buf.move_cursor(Direction::Backward, 1);
                Action::None
            }
            KeyCode::Char('l') | KeyCode::Right => {
                buf.move_cursor(Direction::Forward, 1);
                Action::None
            }
            KeyCode::Char('j') | KeyCode::Down => {
                buf.move_line(Direction::Forward, 1);
                Action::None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                buf.move_line(Direction::Backward, 1);
                Action::None
            }
            _ => Action::None,
        },
    }
}

fn handle_sidebar_key(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            app.sidebar.move_down();
            Action::None
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.sidebar.move_up();
            Action::None
        }
        KeyCode::Enter => {
            if app.sidebar.selected_is_file() {
                if let Some(path) = app.sidebar.selected_path() {
                    let path = path.to_path_buf();
                    app.open_file(&path);
                }
            }
            Action::None
        }
        KeyCode::Tab => {
            app.edit_focus = EditFocus::Editor;
            Action::None
        }
        KeyCode::Char(':') => {
            app.mode = Mode::Command;
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
