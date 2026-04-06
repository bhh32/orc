use orc_bridge::process::{self, ClaudeBridge, OrcEvent};

use clap::{Parser, Subcommand};
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

use std::env;
use std::path::PathBuf;
use std::process as proc;

#[derive(Parser)]
#[command(name = "orc", about = "CLI coding assistant powered by Claude")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Send a single prompt and exit
    #[arg(short, long)]
    prompt: Option<String>,

    /// Model to use
    #[arg(short, long)]
    model: Option<String>,

    /// Effort level (low, medium, high, max)
    #[arg(short, long)]
    effort: Option<String>,

    /// Continue the most recent conversation
    #[arg(short, long, name = "continue")]
    continue_session: bool,

    /// Resume a conversation by session ID
    #[arg(short, long)]
    resume: Option<String>,

    /// Working directory
    #[arg(short = 'C', long)]
    cwd: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
    /// Check Claude Code installation status
    Doctor,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .compact()
        .init();

    if let Err(e) = run().await {
        eprintln!("error: {e}");
        proc::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(Command::Doctor) = &cli.command {
        return run_doctor();
    }

    if !process::is_claude_installed() {
        anyhow::bail!(
            "Claude Code is not installed.\n\
             Install it with: npm install -g @anthropic-ai/claude-code\n\
             Then authenticate with: claude login"
        );
    }

    let cwd = match cli.cwd {
        Some(dir) => dir,
        None => env::current_dir()?,
    };

    let mut bridge = ClaudeBridge::new(cwd);

    if let Some(model) = cli.model {
        bridge = bridge.with_model(model);
    }
    if let Some(effort) = cli.effort {
        bridge.set_effort(effort);
    }
    if let Some(sid) = cli.resume {
        bridge = bridge.with_resume(sid);
    } else if cli.continue_session {
        bridge = bridge.with_continue();
    }

    match cli.prompt {
        Some(prompt) => run_oneshot(bridge, &prompt).await,
        None => orc_tui::run(bridge).await,
    }
}

fn run_doctor() -> anyhow::Result<()> {
    if process::is_claude_installed() {
        println!("  Claude Code: installed");
    } else {
        println!("  Claude Code: NOT FOUND");
        println!("  Install with: npm install -g @anthropic-ai/claude-code");
    }
    Ok(())
}

async fn run_oneshot(mut bridge: ClaudeBridge, prompt: &str) -> anyhow::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel();

    let send_fut = bridge.send(prompt, &tx);
    tokio::pin!(send_fut);

    loop {
        tokio::select! {
            result = &mut send_fut => {
                while let Ok(event) = rx.try_recv() {
                    if let OrcEvent::Text(text) = event {
                        print!("{text}");
                    }
                }
                println!();
                result.map_err(|e| anyhow::anyhow!("{e}"))?;
                break;
            }
            Some(event) = rx.recv() => {
                if let OrcEvent::Text(text) = event {
                    print!("{text}");
                }
            }
        }
    }

    Ok(())
}
