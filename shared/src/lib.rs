//! AuraOS shared types: IPC framing, tool schemas, service names.

pub mod ipc;
pub mod tools;

pub const AURA_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AGENT_SERVICE: &str = "agent.core";
pub const INIT_SERVICE: &str = "init";
pub const SHELL_SERVICE: &str = "shell";
