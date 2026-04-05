pub struct RuleSet {
    auto_allow: Vec<String>,
    deny: Vec<String>,
}

impl RuleSet {
    pub fn new(auto_allow: Vec<String>, deny: Vec<String>) -> Self {
        Self { auto_allow, deny }
    }

    pub fn is_allowed(&self, tool_name: &str) -> bool {
        self.auto_allow.iter().any(|a| a == tool_name || a == "*")
    }

    pub fn is_denied(&self, tool_name: &str) -> bool {
        self.deny.iter().any(|d| d == tool_name || d == "*")
    }
}
