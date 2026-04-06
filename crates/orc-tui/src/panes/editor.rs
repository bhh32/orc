use crate::theme;

use helix_core::doc_formatter::TextFormat;
use helix_core::graphemes;
use helix_core::history::{History, State};
use helix_core::movement::{self, Direction, Movement};
use helix_core::selection::Selection;
use helix_core::text_annotations::TextAnnotations;
use helix_core::{Rope, Tendril, Transaction};

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Normal,
    Insert,
    Select,
    Search,
}

impl EditorMode {
    pub fn label(&self) -> &str {
        match self {
            EditorMode::Normal => "NOR",
            EditorMode::Insert => "INS",
            EditorMode::Select => "SEL",
            EditorMode::Search => "SRC",
        }
    }
}

pub struct Buffer {
    pub rope: Rope,
    pub selection: Selection,
    pub path: Option<PathBuf>,
    pub modified: bool,
    pub mode: EditorMode,
    pub viewport_top: usize,
    history: History,
    pub register: String,
    pub search_query: String,
    pub search_input: String,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            selection: Selection::point(0),
            path: None,
            modified: false,
            mode: EditorMode::Normal,
            viewport_top: 0,
            history: History::default(),
            register: String::new(),
            search_query: String::new(),
            search_input: String::new(),
        }
    }

    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let rope = Rope::from_str(&content);
        Ok(Self {
            rope,
            selection: Selection::point(0),
            path: Some(path.to_path_buf()),
            modified: false,
            mode: EditorMode::Normal,
            viewport_top: 0,
            history: History::default(),
            register: String::new(),
            search_query: String::new(),
            search_input: String::new(),
        })
    }

    pub fn reload(&mut self) -> anyhow::Result<()> {
        if let Some(ref path) = self.path {
            let content = fs::read_to_string(path)?;
            self.rope = Rope::from_str(&content);
            self.modified = false;
            let len = self.rope.len_chars();
            let cursor = self.cursor_pos().min(len.saturating_sub(1));
            self.selection = Selection::point(cursor);
        }
        Ok(())
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        if let Some(ref path) = self.path {
            let content: String = self.rope.chunks().collect();
            fs::write(path, &content)?;
            self.modified = false;
        }
        Ok(())
    }

    pub fn cursor_pos(&self) -> usize {
        self.selection.primary().cursor(self.rope.slice(..))
    }

    pub fn cursor_line(&self) -> usize {
        let pos = self.cursor_pos().min(self.rope.len_chars().saturating_sub(1));
        self.rope.char_to_line(pos)
    }

    pub fn cursor_col(&self) -> usize {
        let pos = self.cursor_pos();
        let line_start = self.rope.line_to_char(self.cursor_line());
        pos - line_start
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn file_name(&self) -> &str {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("[scratch]")
    }

    fn state(&self) -> State {
        State {
            doc: self.rope.clone(),
            selection: self.selection.clone(),
        }
    }

    // Movement

    pub fn move_cursor(&mut self, dir: Direction, count: usize) {
        let text = self.rope.slice(..);
        let range = self.selection.primary();
        let fmt = TextFormat::default();
        let mut ann = TextAnnotations::default();
        let new = movement::move_horizontally(text, range, dir, count, Movement::Move, &fmt, &mut ann);
        self.selection = Selection::single(new.anchor, new.head);
    }

    pub fn move_line(&mut self, dir: Direction, count: usize) {
        let text = self.rope.slice(..);
        let range = self.selection.primary();
        let fmt = TextFormat::default();
        let mut ann = TextAnnotations::default();
        let new = movement::move_vertically(text, range, dir, count, Movement::Move, &fmt, &mut ann);
        self.selection = Selection::single(new.anchor, new.head);
    }

    pub fn extend_cursor(&mut self, dir: Direction, count: usize) {
        let text = self.rope.slice(..);
        let range = self.selection.primary();
        let fmt = TextFormat::default();
        let mut ann = TextAnnotations::default();
        let new = movement::move_horizontally(text, range, dir, count, Movement::Extend, &fmt, &mut ann);
        self.selection = Selection::single(new.anchor, new.head);
    }

    pub fn extend_line(&mut self, dir: Direction, count: usize) {
        let text = self.rope.slice(..);
        let range = self.selection.primary();
        let fmt = TextFormat::default();
        let mut ann = TextAnnotations::default();
        let new = movement::move_vertically(text, range, dir, count, Movement::Extend, &fmt, &mut ann);
        self.selection = Selection::single(new.anchor, new.head);
    }

    pub fn word_next(&mut self) {
        let text = self.rope.slice(..);
        let range = self.selection.primary();
        let new = movement::move_next_word_start(text, range, 1);
        self.selection = Selection::single(new.anchor, new.head);
    }

    pub fn word_prev(&mut self) {
        let text = self.rope.slice(..);
        let range = self.selection.primary();
        let new = movement::move_prev_word_start(text, range, 1);
        self.selection = Selection::single(new.anchor, new.head);
    }

    pub fn word_end(&mut self) {
        let text = self.rope.slice(..);
        let range = self.selection.primary();
        let new = movement::move_next_word_end(text, range, 1);
        self.selection = Selection::single(new.anchor, new.head);
    }

    pub fn goto_line_start(&mut self) {
        let line = self.cursor_line();
        let pos = self.rope.line_to_char(line);
        self.selection = Selection::point(pos);
    }

    pub fn goto_line_end(&mut self) {
        let line = self.cursor_line();
        let end = if line + 1 < self.line_count() {
            self.rope.line_to_char(line + 1).saturating_sub(1)
        } else {
            self.rope.len_chars().saturating_sub(1)
        };
        self.selection = Selection::point(end);
    }

    pub fn goto_file_start(&mut self) {
        self.selection = Selection::point(0);
    }

    pub fn goto_file_end(&mut self) {
        let pos = self.rope.len_chars().saturating_sub(1);
        self.selection = Selection::point(pos);
    }

    pub fn select_line(&mut self) {
        let line = self.cursor_line();
        let start = self.rope.line_to_char(line);
        let end = if line + 1 < self.line_count() {
            self.rope.line_to_char(line + 1)
        } else {
            self.rope.len_chars()
        };
        self.selection = Selection::single(start, end);
    }

    pub fn extend_word_next(&mut self) {
        let text = self.rope.slice(..);
        let range = self.selection.primary();
        let new = movement::move_next_word_start(text, range, 1);
        self.selection = Selection::single(range.anchor, new.head);
    }

    pub fn extend_word_prev(&mut self) {
        let text = self.rope.slice(..);
        let range = self.selection.primary();
        let new = movement::move_prev_word_start(text, range, 1);
        self.selection = Selection::single(range.anchor, new.head);
    }

    pub fn extend_word_end(&mut self) {
        let text = self.rope.slice(..);
        let range = self.selection.primary();
        let new = movement::move_next_word_end(text, range, 1);
        self.selection = Selection::single(range.anchor, new.head);
    }

    pub fn extend_select_line(&mut self) {
        let anchor = self.selection.primary().anchor;
        let line = self.cursor_line();
        let end = if line + 1 < self.line_count() {
            self.rope.line_to_char(line + 1)
        } else {
            self.rope.len_chars()
        };
        self.selection = Selection::single(anchor, end);
    }

    pub fn collapse_selection(&mut self) {
        let pos = self.cursor_pos();
        self.selection = Selection::point(pos);
    }

    // Editing

    pub fn insert_char(&mut self, ch: char) {
        let mut t = Tendril::new();
        t.push(ch);
        let pos = self.cursor_pos();
        let txn = Transaction::insert(&self.rope, &self.selection, t);
        let state = self.state();
        self.history.commit_revision(&txn, &state);
        if txn.apply(&mut self.rope) {
            self.selection = Selection::point(pos + ch.len_utf8());
            self.modified = true;
        }
    }

    pub fn insert_newline(&mut self) {
        self.insert_char('\n');
    }

    pub fn delete_backward(&mut self) {
        let pos = self.cursor_pos();
        if pos == 0 {
            return;
        }
        let prev = graphemes::prev_grapheme_boundary(self.rope.slice(..), pos);
        let txn = Transaction::delete_by_selection(&self.rope, &self.selection, |_| (prev, pos));
        let state = self.state();
        self.history.commit_revision(&txn, &state);
        if txn.apply(&mut self.rope) {
            self.selection = Selection::point(prev);
            self.modified = true;
        }
    }

    pub fn delete_forward(&mut self) {
        let pos = self.cursor_pos();
        if pos >= self.rope.len_chars() {
            return;
        }
        let next = graphemes::next_grapheme_boundary(self.rope.slice(..), pos);
        let state = self.state();
        let txn = Transaction::delete_by_selection(&self.rope, &self.selection, |_| (pos, next));
        self.history.commit_revision(&txn, &state);
        if txn.apply(&mut self.rope) {
            self.selection = Selection::point(pos);
            self.modified = true;
        }
    }

    pub fn delete_selection(&mut self) {
        let text = self.rope.slice(..);
        let txn = Transaction::delete_by_selection(&self.rope, &self.selection, |range| {
            let from = range.from();
            let to = range.to();
            if from == to {
                (from, graphemes::next_grapheme_boundary(text, to))
            } else {
                (from, to)
            }
        });
        let state = self.state();
        self.history.commit_revision(&txn, &state);
        if txn.apply(&mut self.rope) {
            let pos = self.selection.primary().from().min(self.rope.len_chars().saturating_sub(1));
            self.selection = Selection::point(pos);
            self.modified = true;
        }
    }

    pub fn change_selection(&mut self) {
        self.delete_selection();
        self.mode = EditorMode::Insert;
    }

    // Yank / Paste

    pub fn yank(&mut self) {
        let range = self.selection.primary();
        let from = range.from();
        let to = range.to();
        if from == to {
            let line = self.cursor_line();
            let start = self.rope.line_to_char(line);
            let end = if line + 1 < self.line_count() {
                self.rope.line_to_char(line + 1)
            } else {
                self.rope.len_chars()
            };
            self.register = self.rope.slice(start..end).to_string();
        } else {
            self.register = self.rope.slice(from..to).to_string();
        }
    }

    pub fn paste_after(&mut self) {
        if self.register.is_empty() {
            return;
        }
        let pos = self.cursor_pos();
        let insert_at = if self.register.ends_with('\n') {
            let line = self.cursor_line();
            if line + 1 < self.line_count() {
                self.rope.line_to_char(line + 1)
            } else {
                self.rope.len_chars()
            }
        } else {
            graphemes::next_grapheme_boundary(self.rope.slice(..), pos)
        };

        let sel = Selection::point(insert_at);
        let t: Tendril = self.register.as_str().into();
        let txn = Transaction::insert(&self.rope, &sel, t);
        let state = self.state();
        self.history.commit_revision(&txn, &state);
        if txn.apply(&mut self.rope) {
            self.selection = Selection::point(insert_at);
            self.modified = true;
        }
    }

    pub fn paste_before(&mut self) {
        if self.register.is_empty() {
            return;
        }
        let pos = self.cursor_pos();
        let insert_at = if self.register.ends_with('\n') {
            self.rope.line_to_char(self.cursor_line())
        } else {
            pos
        };

        let sel = Selection::point(insert_at);
        let t: Tendril = self.register.as_str().into();
        let txn = Transaction::insert(&self.rope, &sel, t);
        let state = self.state();
        self.history.commit_revision(&txn, &state);
        if txn.apply(&mut self.rope) {
            self.selection = Selection::point(insert_at);
            self.modified = true;
        }
    }

    // Undo / Redo

    pub fn undo(&mut self) {
        if let Some(txn) = self.history.undo() {
            let txn = txn.clone();
            txn.apply(&mut self.rope);
            if let Some(sel) = txn.selection() {
                self.selection = sel.clone();
            }
            self.modified = true;
        }
    }

    pub fn redo(&mut self) {
        if let Some(txn) = self.history.redo() {
            let txn = txn.clone();
            txn.apply(&mut self.rope);
            if let Some(sel) = txn.selection() {
                self.selection = sel.clone();
            }
            self.modified = true;
        }
    }

    // Search

    pub fn search_next(&mut self) {
        if self.search_query.is_empty() {
            return;
        }
        let text: String = self.rope.chunks().collect();
        let start = self.cursor_pos() + 1;
        if let Some(idx) = text[start..].find(&self.search_query) {
            let pos = start + idx;
            let end = pos + self.search_query.len();
            self.selection = Selection::single(pos, end);
        } else if let Some(idx) = text.find(&self.search_query) {
            let end = idx + self.search_query.len();
            self.selection = Selection::single(idx, end);
        }
    }

    pub fn search_prev(&mut self) {
        if self.search_query.is_empty() {
            return;
        }
        let text: String = self.rope.chunks().collect();
        let end = self.cursor_pos();
        if let Some(idx) = text[..end].rfind(&self.search_query) {
            let end = idx + self.search_query.len();
            self.selection = Selection::single(idx, end);
        } else if let Some(idx) = text.rfind(&self.search_query) {
            let end = idx + self.search_query.len();
            self.selection = Selection::single(idx, end);
        }
    }

    // Shell pipe

    pub fn pipe_selection(&mut self, cmd: &str) -> Result<(), String> {
        let range = self.selection.primary();
        let from = range.from();
        let to = range.to();

        let input = if from == to {
            let line = self.cursor_line();
            let start = self.rope.line_to_char(line);
            let end = if line + 1 < self.line_count() {
                self.rope.line_to_char(line + 1)
            } else {
                self.rope.len_chars()
            };
            self.rope.slice(start..end).to_string()
        } else {
            self.rope.slice(from..to).to_string()
        };

        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(ref mut stdin) = child.stdin {
                    stdin.write_all(input.as_bytes()).ok();
                }
                drop(child.stdin.take());
                child.wait_with_output()
            })
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(err);
        }

        let result = String::from_utf8_lossy(&output.stdout).to_string();

        if from == to {
            let line = self.cursor_line();
            let start = self.rope.line_to_char(line);
            let end = if line + 1 < self.line_count() {
                self.rope.line_to_char(line + 1)
            } else {
                self.rope.len_chars()
            };
            let sel = Selection::single(start, end);
            let txn = Transaction::delete_by_selection(&self.rope, &sel, |r| (r.from(), r.to()));
            let state = self.state();
            self.history.commit_revision(&txn, &state);
            txn.apply(&mut self.rope);

            let sel = Selection::point(start);
            let t: Tendril = result.as_str().into();
            let txn = Transaction::insert(&self.rope, &sel, t);
            let state = self.state();
            self.history.commit_revision(&txn, &state);
            txn.apply(&mut self.rope);
            self.selection = Selection::point(start);
        } else {
            let txn = Transaction::delete_by_selection(&self.rope, &self.selection, |r| (r.from(), r.to()));
            let state = self.state();
            self.history.commit_revision(&txn, &state);
            txn.apply(&mut self.rope);

            let sel = Selection::point(from);
            let t: Tendril = result.as_str().into();
            let txn = Transaction::insert(&self.rope, &sel, t);
            let state = self.state();
            self.history.commit_revision(&txn, &state);
            txn.apply(&mut self.rope);
            self.selection = Selection::point(from);
        }

        self.modified = true;
        Ok(())
    }

    pub fn scroll_to_cursor(&mut self, viewport_height: usize) {
        let line = self.cursor_line();
        if line < self.viewport_top {
            self.viewport_top = line;
        } else if line >= self.viewport_top + viewport_height {
            self.viewport_top = line.saturating_sub(viewport_height) + 1;
        }
    }
}

pub fn render(f: &mut Frame, buf: &Buffer, area: Rect, focused: bool) {
    let border_style = if focused {
        theme::border_focused()
    } else {
        theme::border_unfocused()
    };

    let title = if buf.modified {
        format!(" {} [+] ", buf.file_name())
    } else {
        format!(" {} ", buf.file_name())
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(title, border_style));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let gutter_width = 4u16;
    let height = inner.height as usize;

    let mut lines: Vec<Line> = Vec::new();
    let cursor_line = buf.cursor_line();
    let cursor_col = buf.cursor_col();
    let sel_from = buf.selection.primary().from();
    let sel_to = buf.selection.primary().to();
    let has_selection = sel_from != sel_to;

    for i in 0..height {
        let line_idx = buf.viewport_top + i;
        if line_idx >= buf.line_count() {
            let gutter = Span::styled(format!("{:>3} ", "~"), theme::gutter());
            lines.push(Line::from(vec![gutter]));
            continue;
        }

        let line_num = line_idx + 1;
        let gutter_style = if line_idx == cursor_line {
            theme::chat_user()
        } else {
            theme::gutter()
        };
        let gutter = Span::styled(format!("{line_num:>3} "), gutter_style);

        let rope_line = buf.rope.line(line_idx);
        let raw: String = rope_line.chars().collect();
        let text = raw.trim_end_matches('\n').to_string();

        if has_selection {
            let line_start = buf.rope.line_to_char(line_idx);
            let line_end = line_start + text.len();

            if sel_to <= line_start || sel_from >= line_end {
                lines.push(Line::from(vec![gutter, Span::styled(text, theme::chat_assistant())]));
            } else {
                let sel_start_in_line = sel_from.saturating_sub(line_start).min(text.len());
                let sel_end_in_line = sel_to.saturating_sub(line_start).min(text.len());

                let mut spans = vec![gutter];
                if sel_start_in_line > 0 {
                    spans.push(Span::styled(text[..sel_start_in_line].to_string(), theme::chat_assistant()));
                }
                spans.push(Span::styled(text[sel_start_in_line..sel_end_in_line].to_string(), theme::selection()));
                if sel_end_in_line < text.len() {
                    spans.push(Span::styled(text[sel_end_in_line..].to_string(), theme::chat_assistant()));
                }
                lines.push(Line::from(spans));
            }
        } else {
            lines.push(Line::from(vec![gutter, Span::styled(text, theme::chat_assistant())]));
        }
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);

    if focused {
        let x = inner.x + gutter_width + cursor_col as u16;
        let y = inner.y + (cursor_line.saturating_sub(buf.viewport_top)) as u16;
        if y < inner.y + inner.height && x < inner.x + inner.width {
            f.set_cursor_position((x, y));
        }
    }
}
