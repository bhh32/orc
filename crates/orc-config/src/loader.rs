use crate::settings::HookSettings;
use crate::settings::PermissionSettings;
use crate::settings::Settings;

use serde_json::Value;
use tracing::debug;
use tracing::warn;

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub fn load() -> Settings {
    let mut cfg = Settings::default();

    if let Some(global_path) = global_config_path() {
        merge_from_file(&mut cfg, &global_path);
    }

    let project_path = PathBuf::from(".orc/settings.json");
    merge_from_file(&mut cfg, &project_path);

    apply_env_overrides(&mut cfg);

    debug!(model = %cfg.model, max_tokens = cfg.max_tokens, "loaded config");
    cfg
}

fn global_config_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    Some(home.join(".orc/settings.json"))
}

fn merge_from_file(cfg: &mut Settings, path: &Path) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let val: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            warn!(path = %path.display(), error = %e, "invalid config json");
            return;
        }
    };

    debug!(path = %path.display(), "merging config file");
    merge_value_into(cfg, &val);
}

fn merge_value_into(cfg: &mut Settings, val: &Value) {
    if let Some(s) = val.get("model").and_then(Value::as_str) {
        cfg.model = s.to_string();
    }
    if let Some(n) = val.get("max_tokens").and_then(Value::as_u64) {
        cfg.max_tokens = n as u32;
    }
    if let Some(s) = val.get("api_base_url").and_then(Value::as_str) {
        cfg.api_base_url = Some(s.to_string());
    }
    if let Some(n) = val.get("temperature").and_then(Value::as_f64) {
        cfg.temperature = Some(n as f32);
    }
    merge_permissions(cfg, val);
    merge_hooks(cfg, val);
}

fn merge_permissions(cfg: &mut Settings, val: &Value) {
    let Some(perms) = val.get("permissions") else {
        return;
    };

    if let Ok(p) = serde_json::from_value::<PermissionSettings>(perms.clone()) {
        cfg.permissions = p;
    }
}

fn merge_hooks(cfg: &mut Settings, val: &Value) {
    let Some(hooks) = val.get("hooks") else {
        return;
    };

    if let Ok(h) = serde_json::from_value::<HookSettings>(hooks.clone()) {
        cfg.hooks = h;
    }
}

fn apply_env_overrides(cfg: &mut Settings) {
    if let Ok(val) = env::var("ORC_MODEL") {
        cfg.model = val;
    }
    if let Ok(val) = env::var("ORC_MAX_TOKENS") {
        if let Ok(n) = val.parse::<u32>() {
            cfg.max_tokens = n;
        }
    }
    if let Ok(val) = env::var("ORC_API_BASE_URL") {
        cfg.api_base_url = Some(val);
    }
    if let Ok(val) = env::var("ORC_TEMPERATURE") {
        if let Ok(n) = val.parse::<f32>() {
            cfg.temperature = Some(n);
        }
    }
}
