use crate::transport::{Transport, TransportError};
use crate::protocol::JsonRpcMessage;

use serde_json::{json, Value};
use tracing::debug;

pub struct McpClient {
    transport: Box<dyn Transport>,
    next_id: u64,
}

impl McpClient {
    pub fn new(transport: Box<dyn Transport>) -> Self {
        Self {
            transport,
            next_id: 1,
        }
    }

    pub async fn initialize(&mut self) -> Result<Value, TransportError> {
        let msg = JsonRpcMessage::request(
            self.next_id(),
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "orc",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        );

        self.transport.send(msg).await?;
        let resp = self.transport.receive().await?;

        let initialized = JsonRpcMessage::notification("notifications/initialized", json!({}));
        self.transport.send(initialized).await?;

        Ok(resp.result.unwrap_or(Value::Null))
    }

    pub async fn list_tools(&mut self) -> Result<Value, TransportError> {
        let msg = JsonRpcMessage::request(self.next_id(), "tools/list", json!({}));
        self.transport.send(msg).await?;
        let resp = self.transport.receive().await?;
        Ok(resp.result.unwrap_or(Value::Null))
    }

    pub async fn call_tool(
        &mut self,
        name: &str,
        args: Value,
    ) -> Result<Value, TransportError> {
        debug!(tool = name, "calling MCP tool");
        let msg = JsonRpcMessage::request(
            self.next_id(),
            "tools/call",
            json!({ "name": name, "arguments": args }),
        );
        self.transport.send(msg).await?;
        let resp = self.transport.receive().await?;

        if let Some(err) = resp.error {
            return Err(TransportError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                err.message,
            )));
        }

        Ok(resp.result.unwrap_or(Value::Null))
    }

    pub async fn shutdown(&self) -> Result<(), TransportError> {
        self.transport.close().await
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}
