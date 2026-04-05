use crate::protocol::JsonRpcMessage;

use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("connection closed")]
    Closed,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[async_trait]
pub trait Transport: Send + Sync {
    async fn send(&self, msg: JsonRpcMessage) -> Result<(), TransportError>;
    async fn receive(&self) -> Result<JsonRpcMessage, TransportError>;
    async fn close(&self) -> Result<(), TransportError>;
}
