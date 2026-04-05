use crate::traits::Tool;
use crate::traits::ToolContext;
use crate::traits::ToolError;
use crate::traits::ToolOutput;

use async_trait::async_trait;
use serde_json::json;
use serde_json::Value;

use std::fs;
use std::path::Path;

pub struct EditTool;

struct EditParams {
    file_path: String,
    old_string: String,
    new_string: String,
    replace_all: bool,
}

fn extract_params(input: &Value) -> Result<EditParams, ToolError> {
    let file_path = input["file_path"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidInput("file_path is required".into()))?
        .to_string();
    let old_string = input["old_string"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidInput("old_string is required".into()))?
        .to_string();
    let new_string = input["new_string"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidInput("new_string is required".into()))?
        .to_string();
    let replace_all = input["replace_all"].as_bool().unwrap_or(false);
    Ok(EditParams {
        file_path,
        old_string,
        new_string,
        replace_all,
    })
}

fn apply_edit(content: &str, params: &EditParams) -> Result<String, ToolError> {
    let count = content.matches(&params.old_string).count();
    if count == 0 {
        return Err(ToolError::StringNotFound);
    }
    if !params.replace_all && count > 1 {
        return Err(ToolError::AmbiguousMatch);
    }
    if params.replace_all {
        Ok(content.replace(&params.old_string, &params.new_string))
    } else {
        Ok(content.replacen(&params.old_string, &params.new_string, 1))
    }
}

#[async_trait]
impl Tool for EditTool {
    fn name(&self) -> &str {
        "edit"
    }

    fn description(&self) -> &str {
        "Performs exact string replacement in a file."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute path to the file to edit"
                },
                "old_string": {
                    "type": "string",
                    "description": "The exact text to find and replace"
                },
                "new_string": {
                    "type": "string",
                    "description": "The replacement text"
                },
                "replace_all": {
                    "type": "boolean",
                    "description": "Replace all occurrences instead of requiring a unique match",
                    "default": false
                }
            },
            "required": ["file_path", "old_string", "new_string"]
        })
    }

    async fn run(&self, input: Value, _ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let params = extract_params(&input)?;
        let path = Path::new(&params.file_path);
        let content =
            fs::read_to_string(path).map_err(|_| ToolError::FileNotFound(path.into()))?;
        let updated = apply_edit(&content, &params)?;
        fs::write(path, &updated)?;
        Ok(ToolOutput::success(format!(
            "Applied edit to {}",
            params.file_path
        )))
    }
}
