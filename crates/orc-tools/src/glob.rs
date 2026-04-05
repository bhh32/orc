use crate::traits::Tool;
use crate::traits::ToolContext;
use crate::traits::ToolError;
use crate::traits::ToolOutput;

use async_trait::async_trait;
use globset::GlobBuilder;
use globset::GlobMatcher;
use ignore::WalkBuilder;
use serde_json::json;
use serde_json::Value;

use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

pub struct GlobTool;

fn extract_params(input: &Value, cwd: &Path) -> Result<(String, PathBuf), ToolError> {
    let pattern = input["pattern"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidInput("pattern is required".into()))?
        .to_string();
    let path = input["path"]
        .as_str()
        .map(PathBuf::from)
        .unwrap_or_else(|| cwd.to_path_buf());
    Ok((pattern, path))
}

fn build_matcher(pattern: &str) -> Result<GlobMatcher, ToolError> {
    GlobBuilder::new(pattern)
        .literal_separator(false)
        .build()
        .map(|g| g.compile_matcher())
        .map_err(|e| ToolError::InvalidInput(format!("invalid glob pattern: {e}")))
}

fn collect_matches(root: &Path, matcher: &GlobMatcher) -> Vec<(PathBuf, SystemTime)> {
    let walker = WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .build();

    let mut matches: Vec<(PathBuf, SystemTime)> = Vec::new();
    for entry in walker.flatten() {
        let path = entry.path();
        let relative = path.strip_prefix(root).unwrap_or(path);
        if !matcher.is_match(relative) && !matcher.is_match(path) {
            continue;
        }
        let mtime = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        matches.push((path.to_path_buf(), mtime));
    }
    matches
}

fn sort_by_mtime(matches: &mut [(PathBuf, SystemTime)]) {
    matches.sort_by(|a, b| b.1.cmp(&a.1));
}

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "Finds files matching a glob pattern, sorted by modification time."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern to match files (e.g. '**/*.rs')"
                },
                "path": {
                    "type": "string",
                    "description": "Root directory to search in (defaults to cwd)"
                }
            },
            "required": ["pattern"]
        })
    }

    async fn run(&self, input: Value, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let (pattern, root) = extract_params(&input, &ctx.cwd)?;
        let matcher = build_matcher(&pattern)?;
        let mut matches = collect_matches(&root, &matcher);
        sort_by_mtime(&mut matches);
        let paths: Vec<String> = matches.iter().map(|(p, _)| p.display().to_string()).collect();
        Ok(ToolOutput::success(paths.join("\n")))
    }
}
