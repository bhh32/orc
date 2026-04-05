use async_trait::async_trait;
use serde_json::Value;
use thiserror::Error;

use std::path::PathBuf;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("file not found: {0}")]
    FileNotFound(PathBuf),

    #[error("string not found in file")]
    StringNotFound,

    #[error("ambiguous match: old_string appears multiple times")]
    AmbiguousMatch,

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("command timed out after {0}ms")]
    Timeout(u64),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub struct ToolContext {
    pub cwd: PathBuf,
}

pub struct ToolOutput {
    pub content: String,
    pub is_error: bool,
}

impl ToolOutput {
    pub fn success(content: String) -> Self {
        Self {
            content,
            is_error: false,
        }
    }

    pub fn error(content: String) -> Self {
        Self {
            content,
            is_error: true,
        }
    }
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> Value;
    async fn run(&self, input: Value, ctx: &ToolContext) -> Result<ToolOutput, ToolError>;
}
