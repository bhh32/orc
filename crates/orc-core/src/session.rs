use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionMeta {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub cwd: PathBuf,
    pub model: String,
}

impl SessionMeta {
    pub fn new(cwd: PathBuf, model: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            cwd,
            model,
        }
    }
}
