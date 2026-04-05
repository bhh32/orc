use crate::conversation::Conversation;
use crate::system_prompt::build_system_prompt;

use orc_api::client::ApiClient;
use orc_api::types::{
    ContentBlock, Delta, MessageRequest, StopReason, StreamEvent, Usage,
};
use orc_api::types::ToolDef as ApiToolDef;
use orc_permissions::policy::{PermissionChecker, PermissionResult};
use orc_permissions::prompt::ask_permission;
use orc_tools::registry::ToolRegistry;
use orc_tools::traits::ToolContext;

use anyhow::{Context, Result};
use serde_json::Value;
use tokio::sync::mpsc;
use tracing::warn;

use std::path::PathBuf;

pub struct Agent {
    client: ApiClient,
    conversation: Conversation,
    tools: ToolRegistry,
    permissions: PermissionChecker,
    model: String,
    max_tokens: u32,
    system_prompt: String,
    cwd: PathBuf,
}

pub enum AgentEvent {
    Text(String),
    ToolStart { name: String, id: String },
    ToolResult { id: String, output: String, is_error: bool },
    TurnComplete { stop_reason: StopReason, usage: Usage },
    Error(String),
}

impl Agent {
    pub fn new(
        client: ApiClient,
        tools: ToolRegistry,
        permissions: PermissionChecker,
        model: String,
        max_tokens: u32,
        cwd: PathBuf,
    ) -> Self {
        let system_prompt = build_system_prompt(&cwd);

        Self {
            client,
            conversation: Conversation::new(),
            tools,
            permissions,
            model,
            max_tokens,
            system_prompt,
            cwd,
        }
    }

    pub async fn send(
        &mut self,
        user_input: &str,
        event_tx: &mpsc::UnboundedSender<AgentEvent>,
    ) -> Result<()> {
        self.conversation.push_user(user_input);
        self.run_loop(event_tx).await
    }

    async fn run_loop(
        &mut self,
        event_tx: &mpsc::UnboundedSender<AgentEvent>,
    ) -> Result<()> {
        loop {
            let req = self.build_request();

            let mut rx = self.client.stream(&req).await
                .context("failed to open stream")?;

            let (blocks, stop_reason, usage) = self.consume_stream(&mut rx, event_tx).await?;

            self.conversation.track_usage(&usage);

            if blocks.is_empty() {
                let _ = event_tx.send(AgentEvent::TurnComplete {
                    stop_reason: stop_reason.unwrap_or(StopReason::EndTurn),
                    usage,
                });
                return Ok(());
            }

            self.conversation.push_assistant_blocks(blocks.clone());

            let tool_calls = extract_tool_calls(&blocks);
            if tool_calls.is_empty() {
                let _ = event_tx.send(AgentEvent::TurnComplete {
                    stop_reason: stop_reason.unwrap_or(StopReason::EndTurn),
                    usage,
                });
                return Ok(());
            }

            let results = self.execute_tools(tool_calls, event_tx).await;
            self.conversation.push_tool_results(results);
        }
    }

    async fn consume_stream(
        &self,
        rx: &mut mpsc::Receiver<Result<StreamEvent, orc_api::ApiError>>,
        event_tx: &mpsc::UnboundedSender<AgentEvent>,
    ) -> Result<(Vec<ContentBlock>, Option<StopReason>, Usage)> {
        let mut blocks: Vec<ContentBlock> = Vec::new();
        let mut current_text = String::new();
        let mut current_json = String::new();
        let mut current_tool: Option<(String, String)> = None;
        let mut stop_reason = None;
        let mut usage = Usage::default();

        while let Some(ev) = rx.recv().await {
            match ev {
                Ok(StreamEvent::ContentBlockStart { content_block, .. }) => {
                    current_text.clear();
                    current_json.clear();
                    current_tool = match &content_block {
                        ContentBlock::ToolUse { id, name, .. } => {
                            let _ = event_tx.send(AgentEvent::ToolStart {
                                name: name.clone(),
                                id: id.clone(),
                            });
                            Some((id.clone(), name.clone()))
                        }
                        _ => None,
                    };
                }

                Ok(StreamEvent::ContentBlockDelta { delta, .. }) => match delta {
                    Delta::Text { text } => {
                        let _ = event_tx.send(AgentEvent::Text(text.clone()));
                        current_text.push_str(&text);
                    }
                    Delta::InputJson { partial_json } => {
                        current_json.push_str(&partial_json);
                    }
                },

                Ok(StreamEvent::ContentBlockStop { .. }) => {
                    if !current_text.is_empty() {
                        blocks.push(ContentBlock::Text {
                            text: current_text.clone(),
                        });
                    } else if let Some((id, name)) = current_tool.take() {
                        let input: Value =
                            serde_json::from_str(&current_json).unwrap_or(Value::Null);
                        blocks.push(ContentBlock::ToolUse { id, name, input });
                    }
                }

                Ok(StreamEvent::MessageStart { message }) => {
                    usage = message.usage;
                }

                Ok(StreamEvent::MessageDelta {
                    delta: body,
                    usage: delta_usage,
                }) => {
                    stop_reason = body.stop_reason;
                    if let Some(u) = delta_usage {
                        usage.output_tokens += u.output_tokens;
                    }
                }

                Ok(StreamEvent::MessageStop {}) => break,
                Ok(StreamEvent::Ping {}) => {}

                Ok(StreamEvent::Error { error }) => {
                    let _ = event_tx.send(AgentEvent::Error(error.message.clone()));
                    return Err(anyhow::anyhow!("API error: {}", error.message));
                }

                Err(e) => {
                    warn!(error = %e, "stream error");
                    return Err(e.into());
                }
            }
        }

        Ok((blocks, stop_reason, usage))
    }

    async fn execute_tools(
        &self,
        calls: Vec<ToolCall>,
        event_tx: &mpsc::UnboundedSender<AgentEvent>,
    ) -> Vec<ContentBlock> {
        let mut results = Vec::new();
        let ctx = ToolContext {
            cwd: self.cwd.clone(),
        };

        for call in calls {
            let (content, is_error) = match self.permissions.check(&call.name, &call.input) {
                PermissionResult::Denied { reason } => (reason, true),
                PermissionResult::NeedsPrompt { tool, description } => {
                    match ask_permission(&tool, &description) {
                        Ok(true) => self.run_tool(&call, &ctx).await,
                        Ok(false) => ("tool execution denied by user".into(), true),
                        Err(e) => (format!("permission prompt failed: {e}"), true),
                    }
                }
                PermissionResult::Allowed => self.run_tool(&call, &ctx).await,
            };

            let _ = event_tx.send(AgentEvent::ToolResult {
                id: call.id.clone(),
                output: content.clone(),
                is_error,
            });

            results.push(ContentBlock::ToolResult {
                tool_use_id: call.id,
                content: Some(content),
                is_error: if is_error { Some(true) } else { None },
            });
        }

        results
    }

    async fn run_tool(&self, call: &ToolCall, ctx: &ToolContext) -> (String, bool) {
        match self.tools.get(&call.name) {
            Some(tool) => match tool.run(call.input.clone(), ctx).await {
                Ok(output) => (output.content, output.is_error),
                Err(e) => (e.to_string(), true),
            },
            None => (format!("unknown tool: {}", call.name), true),
        }
    }

    fn build_request(&self) -> MessageRequest {
        let registry_defs = self.tools.schemas();
        let tools = if registry_defs.is_empty() {
            None
        } else {
            let api_defs: Vec<ApiToolDef> = registry_defs
                .into_iter()
                .map(|d| ApiToolDef {
                    name: d.name,
                    description: d.description,
                    input_schema: d.input_schema,
                })
                .collect();
            Some(api_defs)
        };

        MessageRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            messages: self.conversation.messages().to_vec(),
            system: Some(self.system_prompt.clone()),
            tools,
            stream: Some(true),
            temperature: None,
        }
    }

    pub fn conversation(&self) -> &Conversation {
        &self.conversation
    }

    pub fn clear_conversation(&mut self) {
        self.conversation.clear();
    }
}

struct ToolCall {
    id: String,
    name: String,
    input: Value,
}

fn extract_tool_calls(blocks: &[ContentBlock]) -> Vec<ToolCall> {
    blocks
        .iter()
        .filter_map(|b| match b {
            ContentBlock::ToolUse { id, name, input } => Some(ToolCall {
                id: id.clone(),
                name: name.clone(),
                input: input.clone(),
            }),
            _ => None,
        })
        .collect()
}
