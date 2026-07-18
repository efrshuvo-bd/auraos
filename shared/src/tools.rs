//! Built-in Agent Core tools for AuraOS.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
}

pub const BUILTIN_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "system_status",
        description: "Report AuraOS kernel/agent/shell status summary",
    },
    ToolSpec {
        name: "echo",
        description: "Echo back the provided text argument",
    },
    ToolSpec {
        name: "list_services",
        description: "List running AuraOS system services",
    },
    ToolSpec {
        name: "help",
        description: "Describe available agent tools and usage",
    },
];

pub fn find_tool(name: &str) -> Option<&'static ToolSpec> {
    BUILTIN_TOOLS.iter().find(|t| t.name == name)
}
