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
    // Picker intercepts all input when visible
    if app.picker.visible {
        return handle_picker(app, key);
    }

    if key.code == KeyCode::BackTab {
        app.permission_mode = app.permission_mode.cycle();
        app.push_system(&format!("mode: {}", app.permission_mode.label()));
        return Action::None;
    }

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

fn handle_picker(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => {
            app.picker.close();
        }
        KeyCode::Enter => {
            if let Some(path) = app.picker.selected_path() {
                let cwd = std::env::current_dir().unwrap_or_default();
                let full = cwd.join(path);
                app.open_file(&full);
                app.view = AppView::Edit;
            }
            app.picker.close();
        }
        KeyCode::Up | KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.picker.move_up();
        }
        KeyCode::Down | KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.picker.move_down();
        }
        KeyCode::Up => app.picker.move_up(),
        KeyCode::Down => app.picker.move_down(),
        KeyCode::Backspace => app.picker.backspace(),
        KeyCode::Char(ch) => app.picker.type_char(ch),
        _ => {}
    }
    Action::None
}

// -- Chat mode --

fn handle_chat_key(app: &mut App, key: KeyEvent) -> Action {
    match app.mode {
        Mode::Normal => handle_chat_normal(app, key),
        Mode::Insert => handle_chat_insert(app, key),
        Mode::Command => handle_command(app, key),
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
            app.scroll_to_bottom();
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

// -- Edit mode --

fn handle_edit_key(app: &mut App, key: KeyEvent) -> Action {
    if app.mode == Mode::Command {
        return handle_command(app, key);
    }

    if app.buffer.mode == EditorMode::Search {
        return handle_search(app, key);
    }

    match app.edit_focus {
        EditFocus::Sidebar => handle_sidebar_key(app, key),
        EditFocus::Editor => handle_editor_key(app, key),
    }
}

fn handle_editor_key(app: &mut App, key: KeyEvent) -> Action {
    let mode = app.buffer.mode;

    match mode {
        EditorMode::Normal => handle_editor_normal(app, key),
        EditorMode::Insert => handle_editor_insert(app, key),
        EditorMode::Select => handle_editor_select(app, key),
        EditorMode::Search => Action::None,
    }
}

fn handle_editor_normal(app: &mut App, key: KeyEvent) -> Action {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('u') => { app.buffer.undo(); return Action::None; }
            KeyCode::Char('r') => { app.buffer.redo(); return Action::None; }
            _ => {}
        }
    }

    match key.code {
        // Mode switches
        KeyCode::Char('i') => { app.buffer.mode = EditorMode::Insert; }
        KeyCode::Char('a') => {
            app.buffer.move_cursor(Direction::Forward, 1);
            app.buffer.mode = EditorMode::Insert;
        }
        KeyCode::Char('I') => {
            app.buffer.goto_line_start();
            app.buffer.mode = EditorMode::Insert;
        }
        KeyCode::Char('A') => {
            app.buffer.goto_line_end();
            app.buffer.mode = EditorMode::Insert;
        }
        KeyCode::Char('o') => {
            app.buffer.goto_line_end();
            app.buffer.insert_newline();
            app.buffer.mode = EditorMode::Insert;
        }
        KeyCode::Char('O') => {
            app.buffer.goto_line_start();
            app.buffer.insert_newline();
            app.buffer.move_line(Direction::Backward, 1);
            app.buffer.mode = EditorMode::Insert;
        }
        KeyCode::Char('v') => { app.buffer.mode = EditorMode::Select; }

        // Movement
        KeyCode::Char('h') | KeyCode::Left  => app.buffer.move_cursor(Direction::Backward, 1),
        KeyCode::Char('l') | KeyCode::Right => app.buffer.move_cursor(Direction::Forward, 1),
        KeyCode::Char('j') | KeyCode::Down  => app.buffer.move_line(Direction::Forward, 1),
        KeyCode::Char('k') | KeyCode::Up    => app.buffer.move_line(Direction::Backward, 1),

        // Word motions
        KeyCode::Char('w') => app.buffer.word_next(),
        KeyCode::Char('b') => app.buffer.word_prev(),
        KeyCode::Char('e') => app.buffer.word_end(),

        // Line
        KeyCode::Char('0') | KeyCode::Home => app.buffer.goto_line_start(),
        KeyCode::Char('$') | KeyCode::End  => app.buffer.goto_line_end(),

        // File
        KeyCode::Char('g') => app.buffer.goto_file_start(),
        KeyCode::Char('G') => app.buffer.goto_file_end(),

        // Select line (Helix 'x')
        KeyCode::Char('x') => app.buffer.select_line(),

        // Edit
        KeyCode::Char('d') => app.buffer.delete_selection(),
        KeyCode::Char('c') => app.buffer.change_selection(),

        // Yank / Paste
        KeyCode::Char('y') => app.buffer.yank(),
        KeyCode::Char('p') => app.buffer.paste_after(),
        KeyCode::Char('P') => app.buffer.paste_before(),

        // Undo / Redo
        KeyCode::Char('u') => app.buffer.undo(),
        KeyCode::Char('U') => app.buffer.redo(),

        // Search
        KeyCode::Char('/') => {
            app.buffer.search_input.clear();
            app.buffer.mode = EditorMode::Search;
        }
        KeyCode::Char('n') => app.buffer.search_next(),
        KeyCode::Char('N') => app.buffer.search_prev(),

        // File picker (Space+f)
        KeyCode::Char(' ') => {
            // Helix space menu — for now just 'f' is handled inline
            // We consume space and wait for next key, but since we can't
            // buffer keys easily, open picker directly on space
            let cwd = std::env::current_dir().unwrap_or_default();
            app.picker.open(&cwd);
        }

        // Command & focus
        KeyCode::Char(':') => { app.mode = Mode::Command; }
        KeyCode::Tab => { app.edit_focus = EditFocus::Sidebar; }

        _ => {}
    }
    Action::None
}

fn handle_editor_insert(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => { app.buffer.mode = EditorMode::Normal; }
        KeyCode::Enter    => app.buffer.insert_newline(),
        KeyCode::Backspace => app.buffer.delete_backward(),
        KeyCode::Delete => app.buffer.delete_forward(),
        KeyCode::Tab => app.buffer.insert_char('\t'),
        KeyCode::Left  => app.buffer.move_cursor(Direction::Backward, 1),
        KeyCode::Right => app.buffer.move_cursor(Direction::Forward, 1),
        KeyCode::Up    => app.buffer.move_line(Direction::Backward, 1),
        KeyCode::Down  => app.buffer.move_line(Direction::Forward, 1),
        KeyCode::Home  => app.buffer.goto_line_start(),
        KeyCode::End   => app.buffer.goto_line_end(),
        KeyCode::Char(ch) => app.buffer.insert_char(ch),
        _ => {}
    }
    Action::None
}

fn handle_editor_select(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => {
            app.buffer.mode = EditorMode::Normal;
            let pos = app.buffer.cursor_pos();
            app.buffer.selection = helix_core::selection::Selection::point(pos);
        }

        // Extend selection with movement
        KeyCode::Char('h') | KeyCode::Left  => app.buffer.extend_cursor(Direction::Backward, 1),
        KeyCode::Char('l') | KeyCode::Right => app.buffer.extend_cursor(Direction::Forward, 1),
        KeyCode::Char('j') | KeyCode::Down  => app.buffer.extend_line(Direction::Forward, 1),
        KeyCode::Char('k') | KeyCode::Up    => app.buffer.extend_line(Direction::Backward, 1),

        // Word extend
        KeyCode::Char('w') => app.buffer.extend_word_next(),
        KeyCode::Char('b') => app.buffer.extend_word_prev(),
        KeyCode::Char('e') => app.buffer.extend_word_end(),

        // Select line
        KeyCode::Char('x') => app.buffer.extend_select_line(),

        // Operate on selection
        KeyCode::Char('d') => {
            app.buffer.delete_selection();
            app.buffer.mode = EditorMode::Normal;
        }
        KeyCode::Char('c') => app.buffer.change_selection(),
        KeyCode::Char('y') => {
            app.buffer.yank();
            app.buffer.mode = EditorMode::Normal;
        }

        KeyCode::Char(':') => { app.mode = Mode::Command; }
        _ => {}
    }
    Action::None
}

fn handle_search(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => {
            app.buffer.search_input.clear();
            app.buffer.mode = EditorMode::Normal;
        }
        KeyCode::Enter => {
            let query = app.buffer.search_input.clone();
            if query.starts_with('!') {
                let cmd = query[1..].to_string();
                if let Err(e) = app.buffer.pipe_selection(&cmd) {
                    app.push_error(&e);
                }
                app.buffer.mode = EditorMode::Normal;
            } else {
                app.buffer.search_query = query;
                app.buffer.search_next();
                app.buffer.mode = EditorMode::Normal;
            }
            app.buffer.search_input.clear();
        }
        KeyCode::Backspace => {
            app.buffer.search_input.pop();
        }
        KeyCode::Char(ch) => {
            app.buffer.search_input.push(ch);
        }
        _ => {}
    }
    Action::None
}

// -- Sidebar --

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

// -- Command mode --

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
