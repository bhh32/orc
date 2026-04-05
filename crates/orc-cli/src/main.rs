mod commands;
mod input;
mod render;
mod repl;

use orc_api::client::{ApiClient, AuthHeader};
use orc_config::loader;
use orc_config::settings::Settings;
use orc_core::agent::Agent;
use orc_permissions::policy::PermissionChecker;
use orc_tools::registry::ToolRegistry;

use anyhow::{Context, Result};
use clap::Parser;
use tracing_subscriber::EnvFilter;

use std::env;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "orc", about = "CLI coding assistant powered by Claude")]
struct Cli {
    /// Send a single prompt and exit
    #[arg(short, long)]
    prompt: Option<String>,

    /// API key (overrides environment/config)
    #[arg(long)]
    api_key: Option<String>,

    /// Model to use
    #[arg(short, long)]
    model: Option<String>,

    /// Working directory
    #[arg(short = 'C', long)]
    cwd: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .compact()
        .init();

    let cli = Cli::parse();

    let cwd = match cli.cwd {
        Some(dir) => dir,
        None => env::current_dir().context("failed to get cwd")?,
    };

    let cfg = loader::load();

    let model = cli.model
        .unwrap_or(cfg.model.clone());
    let max_tokens = cfg.max_tokens;

    let auth = resolve_auth(&cli.api_key, &cfg)?;
    let client = build_client(auth, &cfg)?;

    let tools = ToolRegistry::build_default();
    let permissions = PermissionChecker::new(
        cfg.permissions.auto_allow.clone(),
        cfg.permissions.deny.clone(),
    );

    let agent = Agent::new(client, tools, permissions, model, max_tokens, cwd);

    match cli.prompt {
        Some(prompt) => run_oneshot(agent, &prompt).await,
        None => repl::run(agent).await,
    }
}

async fn run_oneshot(mut agent: Agent, prompt: &str) -> Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    agent.send(prompt, &tx).await?;

    drop(tx);
    while let Some(event) = rx.try_recv().ok() {
        match event {
            orc_core::agent::AgentEvent::Text(text) => print!("{text}"),
            orc_core::agent::AgentEvent::Error(msg) => eprintln!("error: {msg}"),
            _ => {}
        }
    }
    println!();

    Ok(())
}

fn resolve_auth(cli_key: &Option<String>, _cfg: &Settings) -> Result<AuthHeader> {
    if let Some(key) = cli_key {
        return Ok(AuthHeader::ApiKey(key.clone()));
    }

    if let Ok(token) = env::var("ANTHROPIC_AUTH_TOKEN") {
        return Ok(AuthHeader::Bearer(token));
    }

    if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
        return Ok(AuthHeader::ApiKey(key));
    }

    anyhow::bail!(
        "no credentials found. Set ANTHROPIC_API_KEY or run `orc login`"
    )
}

fn build_client(auth: AuthHeader, cfg: &Settings) -> Result<ApiClient> {
    match &cfg.api_base_url {
        Some(url) => ApiClient::with_base_url(auth, url)
            .context("failed to create API client"),
        None => ApiClient::new(auth)
            .context("failed to create API client"),
    }
}
