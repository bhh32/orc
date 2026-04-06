use orc_bridge::process::OrcEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Assistant,
    Tool,
    System,
    Error,
}

pub struct App {
    pub mode: Mode,
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
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: Mode::Normal,
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
        }
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

    pub fn push_tool_message(&mut self, name: &str) {
        self.messages.push(ChatMessage {
            role: ChatRole::Tool,
            content: format!("[{name}]"),
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
            }
            OrcEvent::ToolStart { name, .. } => {
                self.push_tool_message(&name);
            }
            OrcEvent::ToolEnd { .. } => {}
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
