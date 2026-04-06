use crate::app::App;
use crate::input::{self, Action};
use crate::ui;

use orc_bridge::process::ClaudeBridge;

use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;

use std::io;
use std::time::Duration;

pub async fn run(mut bridge: ClaudeBridge) -> anyhow::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut bridge).await;

    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    bridge: &mut ClaudeBridge,
) -> anyhow::Result<()> {
    let mut app = App::new();
    app.mode = crate::app::Mode::Insert;
    app.push_system("orc — press Esc for normal mode, :q to quit");

    terminal.draw(|f| ui::render(f, &app))?;

    loop {
        if app.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match input::handle_key(&mut app, key) {
                    Action::SendMessage(text) => {
                        send_message(&mut app, bridge, &text, terminal).await?;
                    }
                    Action::ExecuteCommand(cmd) => {
                        execute_command(&mut app, bridge, &cmd, terminal).await?;
                    }
                    Action::Quit => {
                        app.should_quit = true;
                    }
                    Action::None => {}
                }
            }
        }

        terminal.draw(|f| ui::render(f, &app))?;
    }

    Ok(())
}

async fn send_message(
    app: &mut App,
    bridge: &mut ClaudeBridge,
    text: &str,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> anyhow::Result<()> {
    app.push_user_message(text);
    app.streaming = true;
    terminal.draw(|f| ui::render(f, app))?;

    let (tx, mut rx) = mpsc::unbounded_channel();

    let send_fut = bridge.send(text, &tx);
    tokio::pin!(send_fut);

    loop {
        tokio::select! {
            result = &mut send_fut => {
                while let Ok(ev) = rx.try_recv() {
                    app.handle_bridge_event(ev);
                }
                if let Err(e) = result {
                    app.push_error(&e.to_string());
                }
                app.streaming = false;
                break;
            }
            Some(ev) = rx.recv() => {
                app.handle_bridge_event(ev);
                terminal.draw(|f| ui::render(f, app))?;
            }
        }
    }

    Ok(())
}

async fn execute_command(
    app: &mut App,
    bridge: &mut ClaudeBridge,
    cmd: &str,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> anyhow::Result<()> {
    let parts: Vec<&str> = cmd.splitn(2, char::is_whitespace).collect();
    let name = parts[0];
    let arg = parts.get(1).unwrap_or(&"").trim();

    match name {
        "q" | "quit" => {
            app.should_quit = true;
        }
        "model" => {
            if arg.is_empty() {
                let cur = app.model.as_deref().unwrap_or("default");
                app.push_system(&format!("model: {cur}"));
            } else {
                bridge.set_model(arg.to_string());
                app.model = Some(arg.to_string());
                app.push_system(&format!("model set to: {arg}"));
            }
        }
        "effort" => {
            if arg.is_empty() {
                app.push_system("usage: :effort <low|medium|high|max>");
            } else {
                bridge.set_effort(arg.to_string());
                app.push_system(&format!("effort set to: {arg}"));
            }
        }
        "cost" => {
            let msg = format!(
                "cost: ${:.4}  |  {}↑ {}↓  |  {} turns",
                app.cost_usd, app.input_tokens, app.output_tokens, app.turns,
            );
            app.push_system(&msg);
        }
        "w" | "write" => {
            match app.buffer.save() {
                Ok(()) => app.push_system("saved"),
                Err(e) => app.push_error(&format!("save failed: {e}")),
            }
        }
        "open" | "o" if !arg.is_empty() => {
            let path = std::path::PathBuf::from(arg);
            app.open_file(&path);
            app.view = crate::app::AppView::Edit;
        }
        _ => {
            let full = format!("/{cmd}");
            send_message(app, bridge, &full, terminal).await?;
        }
    }

    Ok(())
}
