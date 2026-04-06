use crate::events::{BridgeEvent, Delta, StreamEvent, SystemEvent};
use crate::session::Session;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tracing::debug;

use std::path::PathBuf;
use std::process::Stdio;

#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("claude command not found — install Claude Code first")]
    NotInstalled,

    #[error("claude process failed: {0}")]
    ProcessFailed(String),

    #[error("failed to parse event: {0}")]
    ParseError(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub enum OrcEvent {
    Text(String),
    ToolStart {
        name: String,
        id: String,
        input: serde_json::Value,
    },
    ToolInput {
        id: String,
        json_chunk: String,
    },
    ToolEnd {
        id: String,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: bool,
    },
    TurnComplete {
        result: String,
        input_tokens: u64,
        output_tokens: u64,
        cost_usd: f64,
        turns: u32,
    },
    SessionInit {
        session_id: String,
        model: String,
        tools: Vec<String>,
        slash_commands: Vec<String>,
    },
    Error(String),
}

pub struct ClaudeBridge {
    cwd: PathBuf,
    model: Option<String>,
    effort: Option<String>,
    continue_last: bool,
    session: Session,
    active_tools: std::collections::HashMap<usize, String>,
}

impl ClaudeBridge {
    pub fn new(cwd: PathBuf) -> Self {
        Self {
            cwd,
            model: None,
            effort: None,
            continue_last: false,
            session: Session::new(),
            active_tools: std::collections::HashMap::new(),
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }

    pub fn with_resume(mut self, session_id: String) -> Self {
        self.session.id = Some(session_id);
        self
    }

    pub fn with_continue(mut self) -> Self {
        self.continue_last = true;
        self
    }

    pub fn set_model(&mut self, model: String) {
        self.model = Some(model);
    }

    pub fn set_effort(&mut self, effort: String) {
        self.effort = Some(effort);
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub async fn send(
        &mut self,
        prompt: &str,
        event_tx: &mpsc::UnboundedSender<OrcEvent>,
    ) -> Result<(), BridgeError> {
        let mut args = vec![
            "-p".to_string(),
            prompt.to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--verbose".to_string(),
            "--include-partial-messages".to_string(),
        ];

        if let Some(ref model) = self.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }

        if let Some(ref effort) = self.effort {
            args.push("--effort".to_string());
            args.push(effort.clone());
        }

        if let Some(ref sid) = self.session.id {
            args.push("--resume".to_string());
            args.push(sid.clone());
        } else if self.continue_last {
            args.push("--continue".to_string());
            self.continue_last = false;
        }

        let mut child = Command::new("claude")
            .args(&args)
            .current_dir(&self.cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    BridgeError::NotInstalled
                } else {
                    BridgeError::Io(e)
                }
            })?;

        let stdout = child.stdout.take()
            .ok_or_else(|| BridgeError::ProcessFailed("no stdout".into()))?;

        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<BridgeEvent>(&line) {
                Ok(event) => self.handle_event(event, event_tx),
                Err(e) => {
                    debug!(line = %line, error = %e, "skipping unparseable event");
                }
            }
        }

        let status = child.wait().await?;
        if !status.success() {
            let stderr = child.stderr.take();
            let msg = if let Some(err) = stderr {
                let mut buf = String::new();
                let mut r = BufReader::new(err);
                let _ = r.read_line(&mut buf).await;
                buf
            } else {
                format!("exit code: {}", status.code().unwrap_or(-1))
            };
            return Err(BridgeError::ProcessFailed(msg));
        }

        Ok(())
    }

    fn handle_event(
        &mut self,
        event: BridgeEvent,
        tx: &mpsc::UnboundedSender<OrcEvent>,
    ) {
        match event {
            BridgeEvent::System(sys) => self.handle_system(sys, tx),

            BridgeEvent::Stream(wrapper) => {
                match wrapper.event {
                    StreamEvent::ContentBlockDelta { index, delta } => {
                        match delta {
                            Delta::Text { text } => {
                                let _ = tx.send(OrcEvent::Text(text));
                            }
                            Delta::InputJson { partial_json } => {
                                if let Some(tool_id) = self.active_tools.get(&index) {
                                    let _ = tx.send(OrcEvent::ToolInput {
                                        id: tool_id.clone(),
                                        json_chunk: partial_json,
                                    });
                                }
                            }
                        }
                    }
                    StreamEvent::ContentBlockStart { index, content_block } => {
                        if let crate::events::ContentBlock::ToolUse { id, name, input } = content_block {
                            self.active_tools.insert(index, id.clone());
                            let _ = tx.send(OrcEvent::ToolStart { name, id, input });
                        }
                    }
                    StreamEvent::ContentBlockStop { index } => {
                        if let Some(tool_id) = self.active_tools.remove(&index) {
                            let _ = tx.send(OrcEvent::ToolEnd { id: tool_id });
                        }
                    }
                    _ => {}
                }
            }

            BridgeEvent::Result(result) => {
                let input_tokens = result.usage.as_ref().map(|u| u.input_tokens).unwrap_or(0);
                let output_tokens = result.usage.as_ref().map(|u| u.output_tokens).unwrap_or(0);
                let cost = result.total_cost_usd.unwrap_or(0.0);

                self.session.track_result(result.num_turns, cost, input_tokens, output_tokens);

                if result.is_error {
                    let _ = tx.send(OrcEvent::Error(result.result.clone()));
                }

                let _ = tx.send(OrcEvent::TurnComplete {
                    result: result.result,
                    input_tokens,
                    output_tokens,
                    cost_usd: cost,
                    turns: result.num_turns,
                });
            }

            BridgeEvent::Assistant(asst) => {
                for block in &asst.message.content {
                    if let crate::events::ContentBlock::ToolResult {
                        tool_use_id,
                        content,
                        is_error,
                    } = block
                    {
                        let _ = tx.send(OrcEvent::ToolResult {
                            tool_use_id: tool_use_id.clone(),
                            content: content.clone().unwrap_or_default(),
                            is_error: is_error.unwrap_or(false),
                        });
                    }
                }
            }

            BridgeEvent::RateLimit(rl) => {
                if rl.rate_limit_info.status != "allowed" {
                    let _ = tx.send(OrcEvent::Error(
                        format!("rate limited: {}", rl.rate_limit_info.status),
                    ));
                }
            }
        }
    }

    fn handle_system(
        &mut self,
        sys: SystemEvent,
        tx: &mpsc::UnboundedSender<OrcEvent>,
    ) {
        match sys.subtype.as_str() {
            "init" => {
                if let Some(ref sid) = sys.session_id {
                    let model = sys.model.clone().unwrap_or_default();
                    let slash_commands = sys.slash_commands.clone();
                    self.session.update_from_init(
                        sid,
                        sys.model.as_deref(),
                        sys.tools.clone(),
                        sys.slash_commands.clone(),
                        sys.skills.clone(),
                        sys.plugins.clone(),
                        sys.agents.clone(),
                    );

                    let _ = tx.send(OrcEvent::SessionInit {
                        session_id: sid.clone(),
                        model,
                        tools: sys.tools,
                        slash_commands,
                    });
                }
            }
            _ => {
                debug!(subtype = %sys.subtype, "system event");
            }
        }
    }

    pub fn clear_session(&mut self) {
        self.session = Session::new();
    }
}

pub fn is_claude_installed() -> bool {
    which_claude().is_some()
}

fn which_claude() -> Option<PathBuf> {
    which::which("claude").ok()
        .or_else(|| {
            // Common install locations
            let home = dirs::home_dir()?;
            let paths = [
                home.join(".claude/local/claude"),
                home.join(".local/bin/claude"),
                PathBuf::from("/usr/local/bin/claude"),
                PathBuf::from("/opt/node22/bin/claude"),
            ];
            paths.into_iter().find(|p| p.exists())
        })
}
