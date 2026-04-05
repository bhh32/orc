use crate::traits::Tool;
use crate::traits::ToolContext;
use crate::traits::ToolError;
use crate::traits::ToolOutput;

use async_trait::async_trait;
use serde_json::json;
use serde_json::Value;
use tokio::process::Command;
use tokio::time::timeout;
use tokio::time::Duration;

pub struct BashTool;

const DEFAULT_TIMEOUT_MS: u64 = 120_000;

fn extract_params(input: &Value) -> Result<(String, u64), ToolError> {
    let command = input["command"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidInput("command is required".into()))?
        .to_string();
    let timeout_ms = input["timeout"].as_u64().unwrap_or(DEFAULT_TIMEOUT_MS);
    Ok((command, timeout_ms))
}

fn combine_output(stdout: &[u8], stderr: &[u8]) -> String {
    let out = String::from_utf8_lossy(stdout);
    let err = String::from_utf8_lossy(stderr);
    let mut combined = String::new();
    if !out.is_empty() {
        combined.push_str(&out);
    }
    if !err.is_empty() {
        if !combined.is_empty() {
            combined.push('\n');
        }
        combined.push_str(&err);
    }
    combined
}

async fn exec(command: &str, cwd: &std::path::Path, timeout_ms: u64) -> Result<String, ToolError> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(cwd)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let duration = Duration::from_millis(timeout_ms);
    match timeout(duration, child.wait()).await {
        Ok(status) => {
            let _status = status?;
            let stdout = read_pipe(child.stdout.take()).await;
            let stderr = read_pipe(child.stderr.take()).await;
            Ok(combine_output(&stdout, &stderr))
        }
        Err(_) => {
            let _ = child.kill().await;
            Err(ToolError::Timeout(timeout_ms))
        }
    }
}

async fn read_pipe<R: tokio::io::AsyncRead + Unpin>(pipe: Option<R>) -> Vec<u8> {
    use tokio::io::AsyncReadExt;
    match pipe {
        Some(mut r) => {
            let mut buf = Vec::new();
            let _ = r.read_to_end(&mut buf).await;
            buf
        }
        None => Vec::new(),
    }
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Executes a shell command and returns its output."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute"
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in milliseconds",
                    "default": 120000
                }
            },
            "required": ["command"]
        })
    }

    async fn run(&self, input: Value, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let (command, timeout_ms) = extract_params(&input)?;
        let output = exec(&command, &ctx.cwd, timeout_ms).await?;
        Ok(ToolOutput::success(output))
    }
}
