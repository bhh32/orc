use crate::theme;

use helix_core::doc_formatter::TextFormat;
use helix_core::graphemes;
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
}

impl EditorMode {
    pub fn label(&self) -> &str {
        match self {
            EditorMode::Normal => "NOR",
            EditorMode::Insert => "INS",
            EditorMode::Select => "SEL",
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

    pub fn move_cursor(&mut self, dir: Direction, count: usize) {
        let text = self.rope.slice(..);
        let range = self.selection.primary();
        let fmt = TextFormat::default();
        let mut annotations = TextAnnotations::default();
        let new_range = movement::move_horizontally(
            text, range, dir, count, Movement::Move, &fmt, &mut annotations,
        );
        self.selection = Selection::single(new_range.anchor, new_range.head);
    }

    pub fn move_line(&mut self, dir: Direction, count: usize) {
        let text = self.rope.slice(..);
        let range = self.selection.primary();
        let fmt = TextFormat::default();
        let mut annotations = TextAnnotations::default();
        let new_range = movement::move_vertically(
            text, range, dir, count, Movement::Move, &fmt, &mut annotations,
        );
        self.selection = Selection::single(new_range.anchor, new_range.head);
    }

    pub fn insert_char(&mut self, ch: char) {
        let pos = self.cursor_pos();
        let mut t = Tendril::new();
        t.push(ch);
        let txn = Transaction::insert(&self.rope, &self.selection, t);
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
        let txn = Transaction::delete_by_selection(&self.rope, &self.selection, |_range| {
            (prev, pos)
        });
        if txn.apply(&mut self.rope) {
            self.selection = Selection::point(prev);
            self.modified = true;
        }
    }

    pub fn delete_selection(&mut self) {
        let txn = Transaction::delete_by_selection(&self.rope, &self.selection, |range| {
            let from = range.from();
            let to = range.to();
            if from == to {
                let next = graphemes::next_grapheme_boundary(self.rope.slice(..), to);
                (from, next)
            } else {
                (from, to)
            }
        });
        if txn.apply(&mut self.rope) {
            let pos = self.selection.primary().from().min(self.rope.len_chars().saturating_sub(1));
            self.selection = Selection::point(pos);
            self.modified = true;
        }
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
        let text: String = rope_line.chars().collect();
        let text = text.trim_end_matches('\n').to_string();

        lines.push(Line::from(vec![
            gutter,
            Span::styled(text, theme::chat_assistant()),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);

    if focused && buf.mode == EditorMode::Insert {
        let x = inner.x + gutter_width + buf.cursor_col() as u16;
        let y = inner.y + (cursor_line - buf.viewport_top) as u16;
        if y < inner.y + inner.height && x < inner.x + inner.width {
            f.set_cursor_position((x, y));
        }
    }
}
