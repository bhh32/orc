use crate::rules::RuleSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    Allow,
    Prompt,
    Deny,
}

#[derive(Debug)]
pub enum PermissionResult {
    Allowed,
    NeedsPrompt { tool: String, description: String },
    Denied { reason: String },
}

pub struct PermissionChecker {
    rules: RuleSet,
}

impl PermissionChecker {
    pub fn new(auto_allow: Vec<String>, deny: Vec<String>) -> Self {
        let rules = RuleSet::new(auto_allow, deny);
        Self { rules }
    }

    pub fn check(&self, tool_name: &str, _input: &Value) -> PermissionResult {
        if self.rules.is_denied(tool_name) {
            return PermissionResult::Denied {
                reason: format!("{tool_name} is blocked by policy"),
            };
        }

        if self.rules.is_allowed(tool_name) {
            return PermissionResult::Allowed;
        }

        PermissionResult::NeedsPrompt {
            tool: tool_name.to_string(),
            description: format!("Allow {tool_name} to execute?"),
        }
    }
}
