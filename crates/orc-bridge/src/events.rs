use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum BridgeEvent {
    #[serde(rename = "system")]
    System(SystemEvent),

    #[serde(rename = "assistant")]
    Assistant(AssistantEvent),

    #[serde(rename = "stream_event")]
    Stream(StreamWrapper),

    #[serde(rename = "result")]
    Result(ResultEvent),

    #[serde(rename = "rate_limit_event")]
    RateLimit(RateLimitEvent),
}

#[derive(Debug, Deserialize)]
pub struct SystemEvent {
    pub subtype: String,
    pub session_id: Option<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Deserialize)]
pub struct AssistantEvent {
    pub message: AssistantMessage,
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
pub struct AssistantMessage {
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,
    pub usage: Option<UsageInfo>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },

    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: Option<String>,
        is_error: Option<bool>,
    },
}

#[derive(Debug, Deserialize)]
pub struct StreamWrapper {
    pub event: StreamEvent,
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: Value },

    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: usize,
        content_block: ContentBlock,
    },

    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { index: usize, delta: Delta },

    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: usize },

    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: MessageDeltaBody,
        usage: Option<UsageInfo>,
    },

    #[serde(rename = "message_stop")]
    MessageStop {},
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Delta {
    #[serde(rename = "text_delta")]
    Text { text: String },

    #[serde(rename = "input_json_delta")]
    InputJson { partial_json: String },
}

#[derive(Debug, Deserialize)]
pub struct MessageDeltaBody {
    pub stop_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsageInfo {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
}

#[derive(Debug, Deserialize)]
pub struct ResultEvent {
    pub subtype: String,
    pub is_error: bool,
    pub result: String,
    pub session_id: String,
    pub stop_reason: Option<String>,
    pub num_turns: u32,
    pub duration_ms: u64,
    pub total_cost_usd: Option<f64>,
    pub usage: Option<UsageInfo>,
}

#[derive(Debug, Deserialize)]
pub struct RateLimitEvent {
    pub rate_limit_info: RateLimitInfo,
}

#[derive(Debug, Deserialize)]
pub struct RateLimitInfo {
    pub status: String,
}
