use crate::error::ApiError;
use crate::types::StreamEvent;

use futures::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use tokio::sync::mpsc;
use tracing::warn;

pub async fn into_stream(mut es: EventSource) -> mpsc::Receiver<Result<StreamEvent, ApiError>> {
    let (tx, rx) = mpsc::channel(64);

    tokio::spawn(async move {
        while let Some(ev) = es.next().await {
            let result = match ev {
                Ok(Event::Open) => continue,
                Ok(Event::Message(msg)) => parse_event(&msg.data),
                Err(e) => {
                    let err = ApiError::Stream(e.to_string());
                    if tx.send(Err(err)).await.is_err() {
                        break;
                    }
                    es.close();
                    break;
                }
            };

            if tx.send(result).await.is_err() {
                break;
            }
        }
    });

    rx
}

fn parse_event(raw: &str) -> Result<StreamEvent, ApiError> {
    serde_json::from_str(raw).map_err(|e| {
        warn!(raw = raw, "failed to parse SSE event");
        ApiError::Serialization(e)
    })
}
