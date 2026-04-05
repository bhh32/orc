use crate::traits::Tool;
use crate::traits::ToolContext;
use crate::traits::ToolError;
use crate::traits::ToolOutput;

use async_trait::async_trait;
use serde_json::json;
use serde_json::Value;

use std::fs;
use std::path::Path;

pub struct WriteTool;

fn extract_params(input: &Value) -> Result<(String, String), ToolError> {
    let file_path = input["file_path"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidInput("file_path is required".into()))?
        .to_string();
    let content = input["content"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidInput("content is required".into()))?
        .to_string();
    Ok((file_path, content))
}

fn write_file(path: &Path, content: &str) -> Result<(), ToolError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

#[async_trait]
impl Tool for WriteTool {
    fn name(&self) -> &str {
        "write"
    }

    fn description(&self) -> &str {
        "Creates or overwrites a file with the given content."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                }
            },
            "required": ["file_path", "content"]
        })
    }

    async fn run(&self, input: Value, _ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let (file_path, content) = extract_params(&input)?;
        let path = Path::new(&file_path);
        write_file(path, &content)?;
        let bytes = content.len();
        Ok(ToolOutput::success(format!("Wrote {bytes} bytes to {file_path}")))
    }
}
