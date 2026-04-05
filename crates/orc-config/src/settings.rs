use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub model: String,
    pub max_tokens: u32,
    pub api_base_url: Option<String>,
    pub temperature: Option<f32>,
    pub permissions: PermissionSettings,
    pub hooks: HookSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionSettings {
    pub auto_allow: Vec<String>,
    pub deny: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HookSettings {
    pub pre_tool: Vec<HookDef>,
    pub post_tool: Vec<HookDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookDef {
    pub tool: Option<String>,
    pub command: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 8192,
            api_base_url: None,
            temperature: None,
            permissions: PermissionSettings::default(),
            hooks: HookSettings::default(),
        }
    }
}
