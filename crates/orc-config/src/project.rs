use tracing::debug;

use std::fs;
use std::path::Path;
use std::path::PathBuf;

static CONTEXT_FILES: &[&str] = &["ORC.md", "CLAUDE.md"];

pub fn load_project_context(start: &Path) -> Option<String> {
    find_context_file(start)
}

fn find_context_file(start: &Path) -> Option<String> {
    let mut dir = start.to_path_buf();

    loop {
        if let Some(content) = try_read_context(&dir) {
            return Some(content);
        }

        if !dir.pop() {
            return None;
        }
    }
}

fn try_read_context(dir: &PathBuf) -> Option<String> {
    for name in CONTEXT_FILES {
        let path = dir.join(name);
        if let Ok(content) = fs::read_to_string(&path) {
            debug!(path = %path.display(), "loaded project context");
            return Some(content);
        }
    }
    None
}
