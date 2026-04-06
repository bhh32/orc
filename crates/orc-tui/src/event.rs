use crate::app::{App, AppView};
use crate::input::{self, Action};
use crate::theme;
use crate::ui;

use orc_bridge::process::{ClaudeBridge, OrcEvent};

use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;

use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub async fn run(bridge: ClaudeBridge) -> anyhow::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let bridge = Arc::new(Mutex::new(bridge));
    let result = run_loop(&mut terminal, bridge).await;

    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    bridge: Arc<Mutex<ClaudeBridge>>,
) -> anyhow::Result<()> {
    let mut app = App::new();
    app.mode = crate::app::Mode::Insert;
    app.push_system("orc — press Esc for normal mode, Ctrl+E for editor, :q to quit");

    let (fs_tx, mut fs_rx) = mpsc::unbounded_channel::<PathBuf>();
    let (bridge_tx, mut bridge_rx) = mpsc::unbounded_channel::<OrcEvent>();

    let _watcher = setup_file_watcher(fs_tx);

    terminal.draw(|f| ui::render(f, &app))?;

    loop {
        if app.should_quit {
            break;
        }

        if app.view == AppView::Edit {
            let h = terminal.size()?.height as usize;
            app.buffer.scroll_to_cursor(h.saturating_sub(4));
        }

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(16)) => {
                while event::poll(Duration::ZERO)? {
                    match event::read()? {
                        Event::Key(key) => {
                            if key.kind != KeyEventKind::Press {
                                continue;
                            }
                            match input::handle_key(&mut app, key) {
                                Action::SendMessage(text) => {
                                    app.push_user_message(&text);
                                    app.streaming = true;
                                    spawn_bridge_send(
                                        bridge.clone(),
                                        text,
                                        bridge_tx.clone(),
                                    );
                                }
                                Action::ExecuteCommand(cmd) => {
                                    execute_command(
                                        &mut app,
                                        &bridge,
                                        &cmd,
                                        &bridge_tx,
                                    );
                                }
                                Action::Quit => {
                                    app.should_quit = true;
                                }
                                Action::None => {}
                            }
                        }
                        Event::Mouse(_) => {}
                        _ => {}
                    }
                }
            }
            Some(event) = bridge_rx.recv() => {
                app.handle_bridge_event(event);
            }
            Some(path) = fs_rx.recv() => {
                if let Some(ref buf_path) = app.buffer.path {
                    if path == *buf_path {
                        let _ = app.buffer.reload();
                    }
                }
            }
        }

        terminal.draw(|f| ui::render(f, &app))?;
    }

    Ok(())
}

fn spawn_bridge_send(
    bridge: Arc<Mutex<ClaudeBridge>>,
    prompt: String,
    tx: mpsc::UnboundedSender<OrcEvent>,
) {
    tokio::spawn(async move {
        // Take bridge out of mutex for the duration of the async send,
        // replace with a temporary. This allows the event loop to continue.
        let mut taken = {
            let mut guard = bridge.lock().unwrap();
            let cwd = std::env::current_dir().unwrap_or_default();
            let mut tmp = ClaudeBridge::new(cwd);
            std::mem::swap(&mut *guard, &mut tmp);
            tmp
        };

        let result = taken.send(&prompt, &tx).await;

        // Put the bridge back
        {
            let mut guard = bridge.lock().unwrap();
            std::mem::swap(&mut *guard, &mut taken);
        }

        if let Err(e) = result {
            let _ = tx.send(OrcEvent::Error(e.to_string()));
        }
    });
}

fn setup_file_watcher(tx: mpsc::UnboundedSender<PathBuf>) -> Option<RecommendedWatcher> {
    let cwd = std::env::current_dir().ok()?;

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                if matches!(
                    event.kind,
                    notify::EventKind::Modify(_) | notify::EventKind::Create(_)
                ) {
                    for path in event.paths {
                        let _ = tx.send(path);
                    }
                }
            }
        },
        Config::default(),
    )
    .ok()?;

    watcher.watch(&cwd, RecursiveMode::Recursive).ok()?;
    Some(watcher)
}

fn execute_command(
    app: &mut App,
    bridge: &Arc<Mutex<ClaudeBridge>>,
    cmd: &str,
    bridge_tx: &mpsc::UnboundedSender<OrcEvent>,
) {
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
                bridge.lock().unwrap().set_model(arg.to_string());
                app.model = Some(arg.to_string());
                app.push_system(&format!("model set to: {arg}"));
            }
        }
        "effort" => {
            if arg.is_empty() {
                app.push_system("usage: :effort <low|medium|high|max>");
            } else {
                bridge.lock().unwrap().set_effort(arg.to_string());
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
        "theme" => {
            if arg.is_empty() {
                let names = theme::available_themes().join(", ");
                app.push_system(&format!("themes: {names}"));
            } else if theme::set_theme(arg) {
                app.push_system(&format!("theme set to: {arg}"));
            } else {
                let names = theme::available_themes().join(", ");
                app.push_error(&format!("unknown theme: {arg} (available: {names})"));
            }
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
            app.push_user_message(&full);
            app.streaming = true;
            spawn_bridge_send(bridge.clone(), full, bridge_tx.clone());
        }
    }
}
