mod commands;
mod input;
mod render;
mod repl;

use orc_api::client::{ApiClient, AuthHeader};
use orc_auth::oauth::OAuthProvider;
use orc_auth::provider::AuthProvider;
use orc_auth::token_store;
use orc_config::loader;
use orc_config::settings::Settings;
use orc_core::agent::Agent;
use orc_permissions::policy::PermissionChecker;
use orc_tools::registry::ToolRegistry;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use std::env;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "orc", about = "CLI coding assistant powered by Claude")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

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

#[derive(Subcommand)]
enum Command {
    /// Authenticate with your Claude subscription via OAuth
    Login,
    /// Remove stored credentials
    Logout,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .compact()
        .init();

    if let Err(e) = run().await {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(cmd) = &cli.command {
        return match cmd {
            Command::Login => run_login().await,
            Command::Logout => run_logout(),
        };
    }

    let cwd = match cli.cwd {
        Some(dir) => dir,
        None => env::current_dir()?,
    };

    let cfg = loader::load();

    let model = cli.model
        .unwrap_or(cfg.model.clone());
    let max_tokens = cfg.max_tokens;

    let auth = resolve_auth(&cli.api_key).await?;
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

async fn run_login() -> anyhow::Result<()> {
    let provider = OAuthProvider::new()
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    provider.login().await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(())
}

fn run_logout() -> anyhow::Result<()> {
    token_store::clear()
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("  Credentials removed.");
    Ok(())
}

async fn run_oneshot(mut agent: Agent, prompt: &str) -> anyhow::Result<()> {
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

async fn resolve_auth(cli_key: &Option<String>) -> anyhow::Result<AuthHeader> {
    if let Some(key) = cli_key {
        return Ok(AuthHeader::ApiKey(key.clone()));
    }

    if let Ok(token) = env::var("ANTHROPIC_AUTH_TOKEN") {
        return Ok(AuthHeader::Bearer(token));
    }

    if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
        return Ok(AuthHeader::ApiKey(key));
    }

    if token_store::credentials_exist() {
        let provider = OAuthProvider::new()
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let token = provider.get_token().await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        return Ok(AuthHeader::Bearer(token.access_token));
    }

    anyhow::bail!(
        "no credentials found. Set ANTHROPIC_API_KEY or run `orc login`"
    )
}

fn build_client(auth: AuthHeader, cfg: &Settings) -> anyhow::Result<ApiClient> {
    match &cfg.api_base_url {
        Some(url) => Ok(ApiClient::with_base_url(auth, url)?),
        None => Ok(ApiClient::new(auth)?),
    }
}
