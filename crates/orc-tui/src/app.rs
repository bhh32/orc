use crate::panes::editor::Buffer;
use crate::panes::picker::FilePicker;
use crate::panes::sidebar::FileTree;

use orc_bridge::process::OrcEvent;

use std::collections::HashMap;
use std::env;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    Chat,
    Edit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditFocus {
    Sidebar,
    Editor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionMode {
    Default,
    Plan,
    AcceptEdits,
}

impl PermissionMode {
    pub fn label(&self) -> &str {
        match self {
            PermissionMode::Default => "default",
            PermissionMode::Plan => "plan",
            PermissionMode::AcceptEdits => "accept edits",
        }
    }

    pub fn cli_flag(&self) -> &str {
        match self {
            PermissionMode::Default => "default",
            PermissionMode::Plan => "plan",
            PermissionMode::AcceptEdits => "acceptEdits",
        }
    }

    pub fn cycle(&self) -> Self {
        match self {
            PermissionMode::Default => PermissionMode::Plan,
            PermissionMode::Plan => PermissionMode::AcceptEdits,
            PermissionMode::AcceptEdits => PermissionMode::Default,
        }
    }
}

impl Mode {
    pub fn label(&self) -> &str {
        match self {
            Mode::Normal => "NOR",
            Mode::Insert => "INS",
            Mode::Command => "CMD",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChatRole {
    User,
    Assistant,
    ToolUse {
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        is_error: bool,
    },
    System,
    Error,
}

pub struct App {
    pub view: AppView,
    pub mode: Mode,
    pub permission_mode: PermissionMode,
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub input_cursor: usize,
    pub command_input: String,
    pub command_cursor: usize,
    pub scroll_offset: u16,
    pub model: Option<String>,
    pub session_id: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_usd: f64,
    pub turns: u32,
    pub streaming: bool,
    pub should_quit: bool,
    pub available_commands: Vec<String>,
    pub buffer: Buffer,
    pub sidebar: FileTree,
    pub edit_focus: EditFocus,
    pub picker: FilePicker,
    pending_tools: HashMap<String, PendingTool>,
}

struct PendingTool {
    name: String,
    json_buf: String,
}

impl App {
    pub fn new() -> Self {
        let cwd = env::current_dir().unwrap_or_default();
        Self {
            view: AppView::Chat,
            mode: Mode::Normal,
            permission_mode: PermissionMode::Default,
            messages: Vec::new(),
            input: String::new(),
            input_cursor: 0,
            command_input: String::new(),
            command_cursor: 0,
            scroll_offset: 0,
            model: None,
            session_id: None,
            input_tokens: 0,
            output_tokens: 0,
            cost_usd: 0.0,
            turns: 0,
            streaming: false,
            should_quit: false,
            available_commands: Vec::new(),
            buffer: Buffer::new(),
            sidebar: FileTree::from_dir(&cwd),
            edit_focus: EditFocus::Editor,
            picker: FilePicker::new(),
            pending_tools: HashMap::new(),
        }
    }

    pub fn toggle_view(&mut self) {
        self.view = match self.view {
            AppView::Chat => {
                self.mode = Mode::Normal;
                if self.buffer.path.is_none() {
                    let cwd = env::current_dir().unwrap_or_default();
                    self.picker.open(&cwd);
                }
                AppView::Edit
            }
            AppView::Edit => {
                self.mode = Mode::Insert;
                AppView::Chat
            }
        };
    }

    pub fn open_file(&mut self, path: &Path) {
        match Buffer::from_file(path) {
            Ok(buf) => {
                self.buffer = buf;
                self.edit_focus = EditFocus::Editor;
            }
            Err(e) => self.push_error(&format!("failed to open {}: {e}", path.display())),
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.chat_line_count();
    }

    pub fn chat_line_count(&self) -> u16 {
        let mut count: u16 = 0;
        for msg in &self.messages {
            count = count.saturating_add(msg.content.lines().count().max(1) as u16 + 1);
        }
        count
    }

    pub fn push_user_message(&mut self, text: &str) {
        self.messages.push(ChatMessage {
            role: ChatRole::User,
            content: text.to_string(),
        });
    }

    pub fn append_assistant_text(&mut self, text: &str) {
        if let Some(last) = self.messages.last_mut() {
            if last.role == ChatRole::Assistant {
                last.content.push_str(text);
                return;
            }
        }
        self.messages.push(ChatMessage {
            role: ChatRole::Assistant,
            content: text.to_string(),
        });
    }

    pub fn push_tool_use(&mut self, name: &str, input: serde_json::Value) {
        let summary = tool_summary(name, &input);
        self.messages.push(ChatMessage {
            role: ChatRole::ToolUse {
                name: name.to_string(),
                input,
            },
            content: summary,
        });
    }

    pub fn push_tool_result(&mut self, content: &str, is_error: bool) {
        self.messages.push(ChatMessage {
            role: ChatRole::ToolResult { is_error },
            content: content.to_string(),
        });
    }

    pub fn push_error(&mut self, msg: &str) {
        self.messages.push(ChatMessage {
            role: ChatRole::Error,
            content: msg.to_string(),
        });
    }

    pub fn push_system(&mut self, msg: &str) {
        self.messages.push(ChatMessage {
            role: ChatRole::System,
            content: msg.to_string(),
        });
    }

    pub fn handle_bridge_event(&mut self, event: OrcEvent) {
        match event {
            OrcEvent::Text(text) => {
                self.streaming = true;
                self.append_assistant_text(&text);
                self.scroll_to_bottom();
            }
            OrcEvent::ToolStart { name, id, .. } => {
                self.pending_tools.insert(id, PendingTool {
                    name,
                    json_buf: String::new(),
                });
            }
            OrcEvent::ToolInput { id, json_chunk } => {
                if let Some(tool) = self.pending_tools.get_mut(&id) {
                    tool.json_buf.push_str(&json_chunk);
                }
            }
            OrcEvent::ToolEnd { id } => {
                if let Some(tool) = self.pending_tools.remove(&id) {
                    let input = serde_json::from_str(&tool.json_buf)
                        .unwrap_or(serde_json::Value::Null);
                    self.push_tool_use(&tool.name, input);
                    self.scroll_to_bottom();
                }
            }
            OrcEvent::ToolResult { content, is_error, .. } => {
                if !content.is_empty() {
                    self.push_tool_result(&content, is_error);
                }
            }
            OrcEvent::TurnComplete { input_tokens, output_tokens, cost_usd, turns, .. } => {
                self.streaming = false;
                self.input_tokens += input_tokens;
                self.output_tokens += output_tokens;
                self.cost_usd += cost_usd;
                self.turns += turns;
            }
            OrcEvent::SessionInit { session_id, model, slash_commands, .. } => {
                self.session_id = Some(session_id);
                self.model = Some(model);
                self.available_commands = slash_commands;
            }
            OrcEvent::Error(msg) => {
                self.streaming = false;
                self.push_error(&msg);
            }
        }
    }

    pub fn take_input(&mut self) -> String {
        let text = self.input.clone();
        self.input.clear();
        self.input_cursor = 0;
        text
    }

    pub fn take_command(&mut self) -> String {
        let text = self.command_input.clone();
        self.command_input.clear();
        self.command_cursor = 0;
        self.mode = Mode::Normal;
        text
    }

    pub fn insert_char(&mut self, ch: char) {
        match self.mode {
            Mode::Insert => {
                self.input.insert(self.input_cursor, ch);
                self.input_cursor += ch.len_utf8();
            }
            Mode::Command => {
                self.command_input.insert(self.command_cursor, ch);
                self.command_cursor += ch.len_utf8();
            }
            Mode::Normal => {}
        }
    }

    pub fn backspace(&mut self) {
        match self.mode {
            Mode::Insert => {
                if self.input_cursor > 0 {
                    let prev = prev_char_boundary(&self.input, self.input_cursor);
                    self.input.drain(prev..self.input_cursor);
                    self.input_cursor = prev;
                }
            }
            Mode::Command => {
                if self.command_cursor > 0 {
                    let prev = prev_char_boundary(&self.command_input, self.command_cursor);
                    self.command_input.drain(prev..self.command_cursor);
                    self.command_cursor = prev;
                }
            }
            Mode::Normal => {}
        }
    }
}

fn prev_char_boundary(s: &str, pos: usize) -> usize {
    let mut idx = pos.saturating_sub(1);
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

fn tool_summary(name: &str, input: &serde_json::Value) -> String {
    let get = |key: &str| input.get(key).and_then(|v| v.as_str()).unwrap_or("");

    match name {
        "Read" => {
            let path = get("file_path");
            if path.is_empty() { name.to_string() } else { path.to_string() }
        }
        "Write" => {
            let path = get("file_path");
            if path.is_empty() { name.to_string() } else { path.to_string() }
        }
        "Edit" => {
            let path = get("file_path");
            if path.is_empty() { name.to_string() } else { path.to_string() }
        }
        "Bash" => {
            let cmd = get("command");
            if cmd.is_empty() { name.to_string() } else { truncate(cmd, 80) }
        }
        "Glob" => {
            let pat = get("pattern");
            if pat.is_empty() { name.to_string() } else { pat.to_string() }
        }
        "Grep" => {
            let pat = get("pattern");
            if pat.is_empty() { name.to_string() } else { pat.to_string() }
        }
        "Agent" => {
            let desc = get("description");
            if desc.is_empty() { name.to_string() } else { desc.to_string() }
        }
        _ => name.to_string(),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}
