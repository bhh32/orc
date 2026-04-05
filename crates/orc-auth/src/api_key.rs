use crate::provider::{AuthError, AuthProvider, AuthToken, AuthType};

use async_trait::async_trait;

pub struct ApiKeyProvider {
    token: String,
    kind: AuthType,
    token_type: String,
}

impl ApiKeyProvider {
    pub fn new(token: String, kind: AuthType, token_type: &str) -> Self {
        Self {
            token,
            kind,
            token_type: token_type.to_string(),
        }
    }
}

#[async_trait]
impl AuthProvider for ApiKeyProvider {
    async fn get_token(&self) -> Result<AuthToken, AuthError> {
        Ok(AuthToken {
            access_token: self.token.clone(),
            token_type: self.token_type.clone(),
            expires_at: None,
        })
    }

    async fn refresh(&self) -> Result<AuthToken, AuthError> {
        self.get_token().await
    }

    fn auth_type(&self) -> AuthType {
        self.kind
    }
}
