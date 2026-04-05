use crate::traits::Tool;
use crate::traits::ToolContext;
use crate::traits::ToolError;
use crate::traits::ToolOutput;

use async_trait::async_trait;
use serde_json::json;
use serde_json::Value;

use std::path::Path;

pub struct ReadTool;

const DEFAULT_LIMIT: usize = 2000;

fn read_with_line_numbers(path: &Path, offset: usize, limit: usize) -> Result<String, ToolError> {
    let content = std::fs::read_to_string(path).map_err(|_| ToolError::FileNotFound(path.into()))?;
    let lines: Vec<&str> = content.lines().collect();
    let start = offset.min(lines.len());
    let end = (start + limit).min(lines.len());
    let numbered: Vec<String> = lines[start..end]
        .iter()
        .enumerate()
        .map(|(i, line)| format!("{}\t{}", start + i + 1, line))
        .collect();
    Ok(numbered.join("\n"))
}

fn extract_params(input: &Value) -> Result<(String, usize, usize), ToolError> {
    let file_path = input["file_path"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidInput("file_path is required".into()))?
        .to_string();
    let offset = input["offset"].as_u64().unwrap_or(0) as usize;
    let limit = input["limit"].as_u64().unwrap_or(DEFAULT_LIMIT as u64) as usize;
    Ok((file_path, offset, limit))
}

#[async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &str {
        "read"
    }

    fn description(&self) -> &str {
        "Reads a file and returns its contents with line numbers."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute path to the file to read"
                },
                "offset": {
                    "type": "integer",
                    "description": "Line offset to start reading from (0-based)",
                    "minimum": 0
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of lines to read",
                    "minimum": 1,
                    "default": 2000
                }
            },
            "required": ["file_path"]
        })
    }

    async fn run(&self, input: Value, _ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let (file_path, offset, limit) = extract_params(&input)?;
        let path = Path::new(&file_path);
        let content = read_with_line_numbers(path, offset, limit)?;
        Ok(ToolOutput::success(content))
    }
}
