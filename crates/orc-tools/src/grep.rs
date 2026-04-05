use crate::traits::Tool;
use crate::traits::ToolContext;
use crate::traits::ToolError;
use crate::traits::ToolOutput;

use async_trait::async_trait;
use ignore::WalkBuilder;
use regex::Regex;
use serde_json::json;
use serde_json::Value;

use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub struct GrepTool;

struct GrepParams {
    pattern: Regex,
    root: PathBuf,
    glob_filter: Option<globset::GlobMatcher>,
    output_mode: OutputMode,
}

enum OutputMode {
    FilesWithMatches,
    Content,
    Count,
}

fn parse_output_mode(s: &str) -> Result<OutputMode, ToolError> {
    match s {
        "files_with_matches" => Ok(OutputMode::FilesWithMatches),
        "content" => Ok(OutputMode::Content),
        "count" => Ok(OutputMode::Count),
        other => Err(ToolError::InvalidInput(format!(
            "unknown output_mode: {other}"
        ))),
    }
}

fn build_glob_filter(pattern: &str) -> Result<globset::GlobMatcher, ToolError> {
    globset::GlobBuilder::new(pattern)
        .literal_separator(false)
        .build()
        .map(|g| g.compile_matcher())
        .map_err(|e| ToolError::InvalidInput(format!("invalid glob filter: {e}")))
}

fn extract_params(input: &Value, cwd: &Path) -> Result<GrepParams, ToolError> {
    let pattern_str = input["pattern"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidInput("pattern is required".into()))?;
    let pattern =
        Regex::new(pattern_str).map_err(|e| ToolError::InvalidInput(format!("bad regex: {e}")))?;
    let root = input["path"]
        .as_str()
        .map(PathBuf::from)
        .unwrap_or_else(|| cwd.to_path_buf());
    let glob_filter = input["glob"]
        .as_str()
        .map(build_glob_filter)
        .transpose()?;
    let mode_str = input["output_mode"].as_str().unwrap_or("files_with_matches");
    let output_mode = parse_output_mode(mode_str)?;
    Ok(GrepParams {
        pattern,
        root,
        glob_filter,
        output_mode,
    })
}

fn should_include(path: &Path, root: &Path, filter: &Option<globset::GlobMatcher>) -> bool {
    let filter = match filter {
        Some(f) => f,
        None => return true,
    };
    let relative = path.strip_prefix(root).unwrap_or(path);
    filter.is_match(relative) || filter.is_match(path)
}

fn walk_files(root: &Path) -> Vec<PathBuf> {
    let walker = WalkBuilder::new(root).hidden(false).git_ignore(true).build();
    walker
        .flatten()
        .filter(|e| e.file_type().is_some_and(|ft| ft.is_file()))
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn search_file(path: &Path, pattern: &Regex) -> Vec<(usize, String)> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    content
        .lines()
        .enumerate()
        .filter(|(_, line)| pattern.is_match(line))
        .map(|(i, line)| (i + 1, line.to_string()))
        .collect()
}

fn format_files_with_matches(results: &[(PathBuf, Vec<(usize, String)>)]) -> String {
    results
        .iter()
        .filter(|(_, matches)| !matches.is_empty())
        .map(|(path, _)| path.display().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_content(results: &[(PathBuf, Vec<(usize, String)>)]) -> String {
    let mut output = Vec::new();
    for (path, matches) in results {
        if matches.is_empty() {
            continue;
        }
        for (line_num, line) in matches {
            output.push(format!("{}:{}: {}", path.display(), line_num, line));
        }
    }
    output.join("\n")
}

fn format_count(results: &[(PathBuf, Vec<(usize, String)>)]) -> String {
    results
        .iter()
        .filter(|(_, matches)| !matches.is_empty())
        .map(|(path, matches)| format!("{}:{}", path.display(), matches.len()))
        .collect::<Vec<_>>()
        .join("\n")
}

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Searches file contents using regex patterns."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Regex pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "File or directory to search in (defaults to cwd)"
                },
                "glob": {
                    "type": "string",
                    "description": "Glob pattern to filter files (e.g. '*.rs')"
                },
                "output_mode": {
                    "type": "string",
                    "enum": ["files_with_matches", "content", "count"],
                    "description": "Output format",
                    "default": "files_with_matches"
                }
            },
            "required": ["pattern"]
        })
    }

    async fn run(&self, input: Value, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let params = extract_params(&input, &ctx.cwd)?;
        let files = walk_files(&params.root);
        let results: Vec<(PathBuf, Vec<(usize, String)>)> = files
            .into_iter()
            .filter(|p| should_include(p, &params.root, &params.glob_filter))
            .map(|p| {
                let matches = search_file(&p, &params.pattern);
                (p, matches)
            })
            .collect();

        let output = match params.output_mode {
            OutputMode::FilesWithMatches => format_files_with_matches(&results),
            OutputMode::Content => format_content(&results),
            OutputMode::Count => format_count(&results),
        };
        Ok(ToolOutput::success(output))
    }
}
