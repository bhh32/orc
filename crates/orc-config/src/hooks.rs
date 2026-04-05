use crate::settings::HookDef;
use crate::settings::HookSettings;

use tokio::process::Command;
use tracing::debug;
use tracing::error;

use std::io;

pub async fn run_pre_hooks(hooks: &HookSettings, tool_name: &str) -> io::Result<()> {
    run_matching_hooks(&hooks.pre_tool, tool_name).await
}

pub async fn run_post_hooks(hooks: &HookSettings, tool_name: &str) -> io::Result<()> {
    run_matching_hooks(&hooks.post_tool, tool_name).await
}

async fn run_matching_hooks(hooks: &[HookDef], tool_name: &str) -> io::Result<()> {
    for hook in hooks.iter().filter(|h| matches_tool(h, tool_name)) {
        run_hook(hook).await?;
    }
    Ok(())
}

fn matches_tool(hook: &HookDef, tool_name: &str) -> bool {
    match &hook.tool {
        None => true,
        Some(t) => t == tool_name,
    }
}

async fn run_hook(hook: &HookDef) -> io::Result<()> {
    debug!(command = %hook.command, "running hook");

    let status = Command::new("sh")
        .arg("-c")
        .arg(&hook.command)
        .status()
        .await?;

    if status.success() {
        return Ok(());
    }

    let code = status.code().unwrap_or(-1);
    error!(command = %hook.command, code, "hook failed");
    Err(io::Error::new(
        io::ErrorKind::Other,
        format!("hook exited with code {code}: {}", hook.command),
    ))
}
