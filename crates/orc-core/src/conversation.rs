use orc_api::types::{ContentBlock, Message, MessageContent, Role, Usage};

pub struct Conversation {
    messages: Vec<Message>,
    total_input: u32,
    total_output: u32,
}

impl Conversation {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            total_input: 0,
            total_output: 0,
        }
    }

    pub fn push_user(&mut self, text: &str) {
        self.messages.push(Message {
            role: Role::User,
            content: MessageContent::Text(text.to_string()),
        });
    }

    pub fn push_assistant_text(&mut self, text: &str) {
        self.messages.push(Message {
            role: Role::Assistant,
            content: MessageContent::Text(text.to_string()),
        });
    }

    pub fn push_assistant_blocks(&mut self, blocks: Vec<ContentBlock>) {
        self.messages.push(Message {
            role: Role::Assistant,
            content: MessageContent::Blocks(blocks),
        });
    }

    pub fn push_tool_results(&mut self, results: Vec<ContentBlock>) {
        self.messages.push(Message {
            role: Role::User,
            content: MessageContent::Blocks(results),
        });
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn track_usage(&mut self, usage: &Usage) {
        self.total_input += usage.input_tokens;
        self.total_output += usage.output_tokens;
    }

    pub fn token_counts(&self) -> (u32, u32) {
        (self.total_input, self.total_output)
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.total_input = 0;
        self.total_output = 0;
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

impl Default for Conversation {
    fn default() -> Self {
        Self::new()
    }
}
