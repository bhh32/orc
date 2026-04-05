use crate::api_key::ApiKeyProvider;
use crate::oauth::OAuthProvider;
use crate::token_store;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthType {
    OAuth,
    ApiKey,
    EnvVar,
}

#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn get_token(&self) -> Result<AuthToken, AuthError>;
    async fn refresh(&self) -> Result<AuthToken, AuthError>;
    fn auth_type(&self) -> AuthType;
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("no credentials found — set ANTHROPIC_API_KEY or run `orc login`")]
    NoCredentials,

    #[error("token expired and refresh failed: {0}")]
    RefreshFailed(String),

    #[error("OAuth not configured — run `orc login` first")]
    OAuthNotConfigured,

    #[error("credential storage error: {0}")]
    Storage(String),
}

pub fn resolve() -> Result<Box<dyn AuthProvider>, AuthError> {
    if let Ok(token) = env::var("ANTHROPIC_AUTH_TOKEN") {
        return Ok(Box::new(ApiKeyProvider::new(token, AuthType::EnvVar, "Bearer")));
    }

    if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
        return Ok(Box::new(ApiKeyProvider::new(key, AuthType::ApiKey, "api-key")));
    }

    if token_store::credentials_exist() {
        let provider = OAuthProvider::new()?;
        return Ok(Box::new(provider));
    }

    Err(AuthError::NoCredentials)
}
