use crate::bash::BashTool;
use crate::edit::EditTool;
use crate::glob::GlobTool;
use crate::grep::GrepTool;
use crate::read::ReadTool;
use crate::traits::Tool;
use crate::write::WriteTool;

use serde::Serialize;
use serde_json::Value;

use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn build_default() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(ReadTool));
        registry.register(Box::new(WriteTool));
        registry.register(Box::new(EditTool));
        registry.register(Box::new(BashTool));
        registry.register(Box::new(GlobTool));
        registry.register(Box::new(GrepTool));
        registry
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    pub fn list(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.tools.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    pub fn schemas(&self) -> Vec<ToolDef> {
        let mut defs: Vec<ToolDef> = self
            .tools
            .values()
            .map(|tool| ToolDef {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                input_schema: tool.schema(),
            })
            .collect();
        defs.sort_by(|a, b| a.name.cmp(&b.name));
        defs
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
