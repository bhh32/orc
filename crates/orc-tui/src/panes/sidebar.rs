use crate::theme;

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use std::fs;
use std::path::{Path, PathBuf};

pub struct FileTree {
    entries: Vec<FileEntry>,
    pub selected: usize,
}

struct FileEntry {
    path: PathBuf,
    name: String,
    is_dir: bool,
    depth: usize,
}

impl FileTree {
    pub fn from_dir(root: &Path) -> Self {
        let mut entries = Vec::new();
        collect_entries(root, root, 0, &mut entries, 3);

        Self {
            entries,
            selected: 0,
        }
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
        }
    }

    pub fn selected_path(&self) -> Option<&Path> {
        self.entries.get(self.selected).map(|e| e.path.as_path())
    }

    pub fn selected_is_file(&self) -> bool {
        self.entries
            .get(self.selected)
            .map(|e| !e.is_dir)
            .unwrap_or(false)
    }
}

fn collect_entries(root: &Path, dir: &Path, depth: usize, out: &mut Vec<FileEntry>, max_depth: usize) {
    if depth > max_depth {
        return;
    }

    let Ok(read_dir) = fs::read_dir(dir) else {
        return;
    };

    let mut items: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
    items.sort_by(|a, b| {
        let a_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let b_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
        b_dir.cmp(&a_dir).then(a.file_name().cmp(&b.file_name()))
    });

    for item in items {
        let name = item.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "target" || name == "node_modules" {
            continue;
        }

        let path = item.path();
        let is_dir = item.file_type().map(|t| t.is_dir()).unwrap_or(false);

        let display = if is_dir {
            format!("{name}/")
        } else {
            name.clone()
        };

        out.push(FileEntry {
            path: path.clone(),
            name: display,
            is_dir,
            depth,
        });

        if is_dir {
            collect_entries(root, &path, depth + 1, out, max_depth);
        }
    }
}

pub fn render(f: &mut Frame, tree: &FileTree, area: Rect, focused: bool) {
    let border_style = if focused {
        theme::border_focused()
    } else {
        theme::border_unfocused()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(" files ", border_style));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let height = inner.height as usize;
    let scroll = if tree.selected >= height {
        tree.selected - height + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();
    for (i, entry) in tree.entries.iter().skip(scroll).take(height).enumerate() {
        let actual_idx = scroll + i;
        let indent = "  ".repeat(entry.depth);
        let icon = if entry.is_dir { "▸ " } else { "  " };
        let label = format!("{indent}{icon}{}", entry.name);

        let style = if actual_idx == tree.selected {
            theme::selection()
        } else if entry.is_dir {
            theme::chat_tool()
        } else {
            theme::chat_dim()
        };

        lines.push(Line::from(Span::styled(label, style)));
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);
}
