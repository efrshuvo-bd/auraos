//! Built-in Agent Core tools for AuraOS.
//!
//! Guest EL0 uses the same names via numeric IDs in `userspace/guest/src/agent_ipc.rs`
//! (in-kernel u64 mailboxes). Host uses these specs over TCP JSON.

use serde::{Deserialize, Serialize};

/// Guest mailbox tool ids (keep in sync with `aura_guest::agent_ipc`).
pub const TOOL_ID_HELP: u64 = 1;
pub const TOOL_ID_SYSTEM_STATUS: u64 = 2;
pub const TOOL_ID_LIST_SERVICES: u64 = 3;
pub const TOOL_ID_ECHO: u64 = 4;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
}

pub const BUILTIN_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "help",
        description: "Describe available agent tools and usage",
    },
    ToolSpec {
        name: "system_status",
        description: "Report AuraOS kernel/agent/shell status summary",
    },
    ToolSpec {
        name: "list_services",
        description: "List running AuraOS system services",
    },
    ToolSpec {
        name: "echo",
        description: "Echo back the provided text argument",
    },
];

pub fn find_tool(name: &str) -> Option<&'static ToolSpec> {
    BUILTIN_TOOLS.iter().find(|t| t.name == name)
}

pub fn tool_id(name: &str) -> Option<u64> {
    match name {
        "help" => Some(TOOL_ID_HELP),
        "system_status" | "status" => Some(TOOL_ID_SYSTEM_STATUS),
        "list_services" => Some(TOOL_ID_LIST_SERVICES),
        "echo" => Some(TOOL_ID_ECHO),
        _ => None,
    }
}
