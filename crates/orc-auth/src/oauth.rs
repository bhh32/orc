use crate::provider::{AuthError, AuthProvider, AuthToken, AuthType};
use crate::token_store::{self, StoredCredentials};

use anthropic_auth::{AsyncOAuthClient, OAuthConfig, OAuthFlow, OAuthMode};
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tracing::{debug, info};

const CALLBACK_PORT: u16 = 1455;

pub struct OAuthProvider {
    client: AsyncOAuthClient,
}

impl OAuthProvider {
    pub fn new() -> Result<Self, AuthError> {
        let config = build_config();
        let client = AsyncOAuthClient::new(config)
            .map_err(|e| AuthError::Storage(format!("failed to create OAuth client: {e}")))?;
        Ok(Self { client })
    }

    pub async fn login(&self) -> Result<AuthToken, AuthError> {
        let flow = self.client.start_flow(OAuthMode::Max)
            .map_err(|e| AuthError::Storage(format!("failed to start OAuth flow: {e}")))?;

        println!("\n  Opening browser to authenticate...");
        println!("  If it doesn't open, visit:\n");
        println!("    {}\n", flow.authorization_url);

        open_browser(&flow.authorization_url);

        let code_with_state = wait_for_auth(&flow).await?;

        let tokens = self.client.exchange_code(
            &code_with_state,
            &flow.state,
            &flow.verifier,
        ).await.map_err(|e| AuthError::RefreshFailed(format!("token exchange failed: {e}")))?;

        let creds = to_stored_creds(&tokens);
        token_store::save(&creds)?;

        info!("authentication successful");
        println!("  Authenticated successfully.\n");

        Ok(to_auth_token(&tokens))
    }
}

async fn wait_for_auth(flow: &OAuthFlow) -> Result<String, AuthError> {
    println!("  Waiting for authorization...");
    println!("  (or paste the code here and press Enter)\n");

    let callback_state = flow.state.clone();
    let callback_fut = tokio::spawn(async move {
        anthropic_auth::run_callback_server(CALLBACK_PORT, &callback_state).await
    });

    let stdin_fut = tokio::spawn(async {
        let reader = BufReader::new(io::stdin());
        let mut lines = reader.lines();
        lines.next_line().await
    });

    tokio::select! {
        result = callback_fut => {
            match result {
                Ok(Ok(cb)) => {
                    let code_with_state = format!("{}#{}", cb.code, cb.state);
                    Ok(code_with_state)
                }
                Ok(Err(e)) => Err(AuthError::Storage(format!("callback failed: {e}"))),
                Err(e) => Err(AuthError::Storage(format!("callback task panicked: {e}"))),
            }
        }
        result = stdin_fut => {
            match result {
                Ok(Ok(Some(line))) => {
                    let trimmed = line.trim().to_string();
                    if trimmed.is_empty() {
                        return Err(AuthError::Storage("no code provided".into()));
                    }
                    Ok(trimmed)
                }
                Ok(Ok(None)) => Err(AuthError::Storage("stdin closed".into())),
                Ok(Err(e)) => Err(AuthError::Storage(format!("stdin read failed: {e}"))),
                Err(e) => Err(AuthError::Storage(format!("stdin task panicked: {e}"))),
            }
        }
    }
}

#[async_trait]
impl AuthProvider for OAuthProvider {
    async fn get_token(&self) -> Result<AuthToken, AuthError> {
        let creds = token_store::load()?;

        if creds.is_expired() {
            debug!("access token expired, refreshing");
            return self.refresh().await;
        }

        Ok(AuthToken {
            access_token: creds.access_token,
            token_type: "Bearer".to_string(),
            expires_at: creds.expires_at,
        })
    }

    async fn refresh(&self) -> Result<AuthToken, AuthError> {
        let creds = token_store::load()?;

        let refresh_tok = creds.refresh_token
            .ok_or_else(|| AuthError::RefreshFailed("no refresh token stored".into()))?;

        debug!("refreshing access token");
        let tokens = self.client.refresh_token(&refresh_tok)
            .await
            .map_err(|e| AuthError::RefreshFailed(e.to_string()))?;

        let new_creds = to_stored_creds(&tokens);
        token_store::save(&new_creds)?;

        debug!("token refreshed successfully");
        Ok(to_auth_token(&tokens))
    }

    fn auth_type(&self) -> AuthType {
        AuthType::OAuth
    }
}

fn build_config() -> OAuthConfig {
    OAuthConfig::builder()
        .redirect_port(CALLBACK_PORT)
        .build()
}

fn to_stored_creds(tokens: &anthropic_auth::TokenSet) -> StoredCredentials {
    let expires_at = Utc.timestamp_opt(tokens.expires_at as i64, 0)
        .single();

    StoredCredentials {
        access_token: tokens.access_token.clone(),
        refresh_token: Some(tokens.refresh_token.clone()),
        expires_at,
        scopes: Vec::new(),
    }
}

fn to_auth_token(tokens: &anthropic_auth::TokenSet) -> AuthToken {
    let expires_at = Utc.timestamp_opt(tokens.expires_at as i64, 0)
        .single();

    AuthToken {
        access_token: tokens.access_token.clone(),
        token_type: "Bearer".to_string(),
        expires_at,
    }
}

fn open_browser(url: &str) {
    if let Err(e) = anthropic_auth::open_browser(url) {
        debug!(error = %e, "failed to open browser automatically");
    }
}
