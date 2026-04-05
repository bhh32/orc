use orc_config::project::load_project_context;

use std::path::Path;

pub fn build_system_prompt(cwd: &Path) -> String {
    let mut parts = Vec::new();

    parts.push(base_prompt());

    if let Some(ctx) = load_project_context(cwd) {
        parts.push(format!("\n# Project Context\n\n{ctx}"));
    }

    let cwd_str = cwd.display();
    parts.push(format!("\n# Environment\n- Working directory: {cwd_str}"));

    parts.join("\n")
}

fn base_prompt() -> String {
    r#"You are orc, a CLI coding assistant powered by Claude. You help users with software engineering tasks including writing code, debugging, refactoring, and answering questions about codebases.

You have access to tools for reading files, writing files, editing files, running shell commands, searching files by pattern, and searching file contents. Use these tools to help the user effectively.

When modifying code:
- Read files before editing them
- Make minimal, targeted changes
- Prefer editing existing files over creating new ones
- Don't add unnecessary comments or documentation"#
        .to_string()
}
