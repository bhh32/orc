use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("JSON serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("SSE stream error: {0}")]
    Stream(String),

    #[error("API returned error {status}: {message}")]
    Response { status: u16, message: String },

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Rate limited, retry after {retry_after_ms}ms")]
    RateLimit { retry_after_ms: u64 },

    #[error("Overloaded, retry after {retry_after_ms}ms")]
    Overloaded { retry_after_ms: u64 },
}
