use crate::commands::{self, SlashCommand};
use crate::input::{InputReader, ReadResult};
use crate::render::Renderer;

use orc_bridge::process::{ClaudeBridge, OrcEvent};

use anyhow::Result;
use tokio::sync::mpsc;

pub async fn run(mut bridge: ClaudeBridge) -> Result<()> {
    let mut input = InputReader::new();
    let mut renderer = Renderer::new();
    let mut available_commands: Vec<String> = Vec::new();

    renderer.print_status("orc — type /help for commands, Ctrl+D to exit");
    renderer.newline();

    loop {
        match input.read_line() {
            ReadResult::Input(line) => {
                if let Some(cmd) = commands::parse(&line) {
                    match cmd {
                        SlashCommand::Exit => break,
                        SlashCommand::Status => show_status(&bridge, &mut renderer),
                        SlashCommand::Help => commands::print_help(&available_commands),
                        SlashCommand::Passthrough(raw) => {
                            renderer.newline();
                            process_turn(&mut bridge, &raw, &mut renderer, &mut available_commands).await?;
                            renderer.newline();
                        }
                    }
                    continue;
                }

                renderer.newline();
                process_turn(&mut bridge, &line, &mut renderer, &mut available_commands).await?;
                renderer.newline();
            }
            ReadResult::Empty => continue,
            ReadResult::Exit => break,
            ReadResult::Error(e) => {
                renderer.print_error(&e);
                break;
            }
        }
    }

    renderer.print_status("bye");
    Ok(())
}

async fn process_turn(
    bridge: &mut ClaudeBridge,
    input: &str,
    renderer: &mut Renderer,
    available_commands: &mut Vec<String>,
) -> Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel();

    let send_fut = bridge.send(input, &tx);
    tokio::pin!(send_fut);

    loop {
        tokio::select! {
            result = &mut send_fut => {
                drain_events(&mut rx, renderer, available_commands);
                if let Err(e) = result {
                    renderer.print_error(&format!("{e}"));
                }
                break;
            }
            Some(event) = rx.recv() => {
                render_event(event, renderer, available_commands);
            }
        }
    }

    Ok(())
}

fn drain_events(
    rx: &mut mpsc::UnboundedReceiver<OrcEvent>,
    renderer: &mut Renderer,
    available_commands: &mut Vec<String>,
) {
    while let Ok(event) = rx.try_recv() {
        render_event(event, renderer, available_commands);
    }
}

fn show_status(bridge: &ClaudeBridge, renderer: &mut Renderer) {
    let session = bridge.session();
    let sid = session.id.as_deref().unwrap_or("none");
    let model = session.model.as_deref().unwrap_or("unknown");

    renderer.print_status(&format!("  session: {sid}"));
    renderer.print_status(&format!("  model: {model}"));
    renderer.print_status(&format!(
        "  tokens: {} in / {} out  |  turns: {}  |  cost: ${:.4}",
        session.total_input_tokens,
        session.total_output_tokens,
        session.turns,
        session.total_cost_usd,
    ));

    if !session.tools.is_empty() {
        renderer.print_status(&format!("  tools: {}", session.tools.len()));
    }
    if !session.plugins.is_empty() {
        renderer.print_status(&format!("  plugins: {}", session.plugins.join(", ")));
    }
    if !session.agents.is_empty() {
        renderer.print_status(&format!("  agents: {}", session.agents.join(", ")));
    }
}

fn render_event(
    event: OrcEvent,
    renderer: &mut Renderer,
    available_commands: &mut Vec<String>,
) {
    match event {
        OrcEvent::Text(text) => renderer.print_assistant_text(&text),
        OrcEvent::ToolStart { name, id } => renderer.print_tool_start(&name, &id),
        OrcEvent::ToolEnd { .. } => {}
        OrcEvent::TurnComplete { input_tokens, output_tokens, cost_usd, .. } => {
            renderer.print_status(&format!(
                "  tokens: {input_tokens} in / {output_tokens} out  |  cost: ${cost_usd:.4}",
            ));
        }
        OrcEvent::SessionInit { model, slash_commands, .. } => {
            renderer.print_status(&format!("  connected: {model}"));
            *available_commands = slash_commands;
        }
        OrcEvent::Error(msg) => renderer.print_error(&msg),
    }
}
