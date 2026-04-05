use crate::provider::{AuthError, AuthProvider, AuthToken, AuthType};
use crate::token_store;

use async_trait::async_trait;

pub struct OAuthProvider {
    _private: (),
}

impl OAuthProvider {
    pub fn new() -> Self {
        Self { _private: () }
    }
}

#[async_trait]
impl AuthProvider for OAuthProvider {
    async fn get_token(&self) -> Result<AuthToken, AuthError> {
        let creds = token_store::load()?;

        if creds.is_expired() {
            return self.refresh().await;
        }

        Ok(AuthToken {
            access_token: creds.access_token,
            token_type: "Bearer".to_string(),
            expires_at: creds.expires_at,
        })
    }

    async fn refresh(&self) -> Result<AuthToken, AuthError> {
        Err(AuthError::OAuthNotConfigured)
    }

    fn auth_type(&self) -> AuthType {
        AuthType::OAuth
    }
}
