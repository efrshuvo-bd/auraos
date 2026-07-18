//! Built-in system tools for Agent Core.

use crate::memory::AgentMemory;
use shared::tools::BUILTIN_TOOLS;
use tokio::sync::Mutex;

pub fn run_help() -> String {
    let mut out = String::from("AuraOS Agent Core tools:\n");
    for t in BUILTIN_TOOLS {
        out.push_str(&format!("  {} — {}\n", t.name, t.description));
    }
    out.push_str("\nShortcuts: help | status | services | echo <text>\n");
    out
}

pub async fn run_system_status(mem: &Mutex<AgentMemory>) -> String {
    let turns = mem.lock().await.turn_count();
    format!(
        "AuraOS status\n  kernel: research v0 (QEMU/host)\n  agent.core: online\n  memory turns: {turns}\n  update window: 4 years (see docs/updates-4y.md)\n"
    )
}

pub fn run_list_services() -> String {
    "services:\n  init\n  agent.core\n  shell\n".into()
}

pub fn run_echo(text: &str) -> String {
    text.to_string()
}
