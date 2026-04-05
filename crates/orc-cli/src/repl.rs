use crate::commands::{self, SlashCommand};
use crate::input::{InputReader, ReadResult};
use crate::render::Renderer;

use orc_core::agent::{Agent, AgentEvent};

use anyhow::Result;
use tokio::sync::mpsc;

pub async fn run(mut agent: Agent) -> Result<()> {
    let mut input = InputReader::new();
    let mut renderer = Renderer::new();

    renderer.print_status("orc — type /help for commands, Ctrl+D to exit");
    renderer.newline();

    loop {
        match input.read_line() {
            ReadResult::Input(line) => {
                if let Some(cmd) = commands::parse(&line) {
                    match cmd {
                        SlashCommand::Help => commands::print_help(),
                        SlashCommand::Clear => {
                            agent.clear_conversation();
                            renderer.print_status("conversation cleared");
                        }
                        SlashCommand::Exit => break,
                        SlashCommand::Compact => {
                            renderer.print_status("compact not yet implemented");
                        }
                        SlashCommand::Unknown(c) => {
                            renderer.print_error(&format!("unknown command: {c}"));
                        }
                    }
                    continue;
                }

                renderer.newline();
                process_turn(&mut agent, &line, &mut renderer).await?;
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

async fn process_turn(agent: &mut Agent, input: &str, renderer: &mut Renderer) -> Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel();

    let send_fut = agent.send(input, &tx);

    tokio::pin!(send_fut);

    loop {
        tokio::select! {
            result = &mut send_fut => {
                drain_events(&mut rx, renderer);
                if let Err(e) = result {
                    renderer.print_error(&format!("{e:#}"));
                }
                break;
            }
            Some(event) = rx.recv() => {
                render_event(event, renderer);
            }
        }
    }

    Ok(())
}

fn drain_events(rx: &mut mpsc::UnboundedReceiver<AgentEvent>, renderer: &mut Renderer) {
    while let Ok(event) = rx.try_recv() {
        render_event(event, renderer);
    }
}

fn render_event(event: AgentEvent, renderer: &mut Renderer) {
    match event {
        AgentEvent::Text(text) => renderer.print_assistant_text(&text),
        AgentEvent::ToolStart { name, id } => renderer.print_tool_start(&name, &id),
        AgentEvent::ToolResult { output, is_error, .. } => {
            renderer.print_tool_result(&output, is_error);
        }
        AgentEvent::TurnComplete { usage, .. } => {
            let msg = format!(
                "  tokens: {} in / {} out",
                usage.input_tokens, usage.output_tokens
            );
            renderer.print_status(&msg);
        }
        AgentEvent::Error(msg) => renderer.print_error(&msg),
    }
}
