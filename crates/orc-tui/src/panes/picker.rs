use crate::theme;

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use std::fs;
use std::path::{Path, PathBuf};

pub struct FilePicker {
    pub visible: bool,
    pub query: String,
    pub matches: Vec<PathBuf>,
    pub selected: usize,
    all_files: Vec<PathBuf>,
}

impl FilePicker {
    pub fn new() -> Self {
        Self {
            visible: false,
            query: String::new(),
            matches: Vec::new(),
            selected: 0,
            all_files: Vec::new(),
        }
    }

    pub fn open(&mut self, root: &Path) {
        self.visible = true;
        self.query.clear();
        self.selected = 0;
        self.all_files = collect_all_files(root, 5);
        self.matches = self.all_files.clone();
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.query.clear();
    }

    pub fn type_char(&mut self, ch: char) {
        self.query.push(ch);
        self.filter();
    }

    pub fn backspace(&mut self) {
        self.query.pop();
        self.filter();
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.matches.len() {
            self.selected += 1;
        }
    }

    pub fn selected_path(&self) -> Option<&Path> {
        self.matches.get(self.selected).map(|p| p.as_path())
    }

    fn filter(&mut self) {
        self.selected = 0;
        if self.query.is_empty() {
            self.matches = self.all_files.clone();
            return;
        }

        let lower_query = self.query.to_lowercase();
        self.matches = self.all_files
            .iter()
            .filter(|p| {
                let name = p.to_string_lossy().to_lowercase();
                fuzzy_match(&name, &lower_query)
            })
            .cloned()
            .collect();
    }
}

fn fuzzy_match(haystack: &str, needle: &str) -> bool {
    let mut needle_chars = needle.chars().peekable();
    for ch in haystack.chars() {
        if needle_chars.peek() == Some(&ch) {
            needle_chars.next();
        }
        if needle_chars.peek().is_none() {
            return true;
        }
    }
    needle_chars.peek().is_none()
}

fn collect_all_files(root: &Path, max_depth: usize) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_files(root, root, 0, max_depth, &mut files);
    files.sort();
    files
}

fn walk_files(root: &Path, dir: &Path, depth: usize, max_depth: usize, out: &mut Vec<PathBuf>) {
    if depth > max_depth {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else { return };

    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "target" || name == "node_modules" {
            continue;
        }

        let path = entry.path();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

        if is_dir {
            walk_files(root, &path, depth + 1, max_depth, out);
        } else {
            if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.to_path_buf());
            }
        }
    }
}

pub fn render(f: &mut Frame, picker: &FilePicker) {
    let area = f.area();

    let popup_width = (area.width * 60 / 100).min(80).max(30);
    let popup_height = (area.height * 60 / 100).min(30).max(10);

    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_focused())
        .title(Span::styled(" open file ", theme::border_focused()));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .split(inner);

    let prompt = Line::from(vec![
        Span::styled(" > ", theme::chat_user()),
        Span::styled(&picker.query, theme::chat_assistant()),
    ]);
    f.render_widget(Paragraph::new(prompt), chunks[0]);

    let visible_height = chunks[1].height as usize;
    let scroll = if picker.selected >= visible_height {
        picker.selected - visible_height + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();
    for (i, path) in picker.matches.iter().skip(scroll).take(visible_height).enumerate() {
        let idx = scroll + i;
        let style = if idx == picker.selected {
            theme::selection()
        } else {
            theme::chat_dim()
        };
        let label = format!("   {}", path.display());
        lines.push(Line::from(Span::styled(label, style)));
    }

    f.render_widget(Paragraph::new(lines), chunks[1]);

    let cursor_x = inner.x + 3 + picker.query.len() as u16;
    f.set_cursor_position((cursor_x, chunks[0].y));
}
