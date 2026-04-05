use crate::error::ApiError;
use crate::streaming;
use crate::types::{MessageRequest, MessageResponse, StreamEvent};

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use reqwest_eventsource::EventSource;
use tokio::sync::mpsc;
use tracing::debug;

const API_BASE: &str = "https://api.anthropic.com";
const API_VERSION: &str = "2023-06-01";

pub struct ApiClient {
    http: Client,
    base_url: String,
    headers: HeaderMap,
}

pub enum AuthHeader {
    Bearer(String),
    ApiKey(String),
}

impl ApiClient {
    pub fn new(auth: AuthHeader) -> Result<Self, ApiError> {
        Self::with_base_url(auth, API_BASE)
    }

    pub fn with_base_url(auth: AuthHeader, base_url: &str) -> Result<Self, ApiError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "anthropic-version",
            HeaderValue::from_static(API_VERSION),
        );

        match auth {
            AuthHeader::Bearer(token) => {
                let val = format!("Bearer {token}");
                headers.insert(AUTHORIZATION, HeaderValue::from_str(&val).unwrap());
            }
            AuthHeader::ApiKey(key) => {
                headers.insert("x-api-key", HeaderValue::from_str(&key).unwrap());
            }
        }

        let http = Client::builder()
            .default_headers(headers.clone())
            .build()?;

        Ok(Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
            headers,
        })
    }

    pub fn update_auth(&mut self, auth: AuthHeader) {
        match auth {
            AuthHeader::Bearer(token) => {
                let val = format!("Bearer {token}");
                self.headers.insert(AUTHORIZATION, HeaderValue::from_str(&val).unwrap());
                self.headers.remove("x-api-key");
            }
            AuthHeader::ApiKey(key) => {
                self.headers.insert("x-api-key", HeaderValue::from_str(&key).unwrap());
                self.headers.remove(AUTHORIZATION);
            }
        }
    }

    pub async fn send(&self, req: &MessageRequest) -> Result<MessageResponse, ApiError> {
        let url = format!("{}/v1/messages", self.base_url);
        debug!(url = %url, model = %req.model, "sending message");

        let resp = self.http
            .post(&url)
            .headers(self.headers.clone())
            .json(req)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(classify_error(status.as_u16(), &body));
        }

        let msg: MessageResponse = resp.json().await?;
        Ok(msg)
    }

    pub async fn stream(
        &self,
        req: &MessageRequest,
    ) -> Result<mpsc::Receiver<Result<StreamEvent, ApiError>>, ApiError> {
        let url = format!("{}/v1/messages", self.base_url);
        debug!(url = %url, model = %req.model, "opening stream");

        let mut stream_req = req.clone();
        stream_req.stream = Some(true);

        let request = self.http
            .post(&url)
            .headers(self.headers.clone())
            .json(&stream_req);

        let es = EventSource::new(request)
            .map_err(|e| ApiError::Stream(e.to_string()))?;

        let rx = streaming::into_stream(es).await;
        Ok(rx)
    }
}

fn classify_error(status: u16, body: &str) -> ApiError {
    match status {
        401 => ApiError::Auth("invalid or expired credentials".into()),
        429 => ApiError::RateLimit { retry_after_ms: 1000 },
        529 => ApiError::Overloaded { retry_after_ms: 5000 },
        _ => ApiError::Response {
            status,
            message: body.to_string(),
        },
    }
}
