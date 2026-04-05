use crate::provider::AuthError;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub scopes: Vec<String>,
}

impl StoredCredentials {
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(exp) => Utc::now() >= exp,
            None => false,
        }
    }
}

fn credentials_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".orc")
        .join(".credentials.json")
}

pub fn credentials_exist() -> bool {
    credentials_path().exists()
}

pub fn load() -> Result<StoredCredentials, AuthError> {
    let path = credentials_path();
    let data = fs::read_to_string(&path)
        .map_err(|e| AuthError::Storage(format!("failed to read {}: {e}", path.display())))?;
    let creds = serde_json::from_str(&data)
        .map_err(|e| AuthError::Storage(format!("invalid credentials file: {e}")))?;
    Ok(creds)
}

pub fn save(creds: &StoredCredentials) -> Result<(), AuthError> {
    let path = credentials_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AuthError::Storage(format!("failed to create {}: {e}", parent.display())))?;
    }

    let json = serde_json::to_string_pretty(creds)
        .map_err(|e| AuthError::Storage(format!("serialization failed: {e}")))?;

    fs::write(&path, &json)
        .map_err(|e| AuthError::Storage(format!("failed to write {}: {e}", path.display())))?;

    set_permissions(&path)?;
    Ok(())
}

pub fn clear() -> Result<(), AuthError> {
    let path = credentials_path();
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| AuthError::Storage(format!("failed to remove {}: {e}", path.display())))?;
    }
    Ok(())
}

#[cfg(unix)]
fn set_permissions(path: &std::path::Path) -> Result<(), AuthError> {
    use std::os::unix::fs::PermissionsExt;

    let perms = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, perms)
        .map_err(|e| AuthError::Storage(format!("failed to set permissions: {e}")))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_permissions(_path: &std::path::Path) -> Result<(), AuthError> {
    Ok(())
}
