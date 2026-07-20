//! AuraOS shared types: IPC framing, tool schemas, service names, OTA metadata.

pub mod ipc;
pub mod ota;
pub mod tools;
pub mod trust;

pub const AURA_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AGENT_SERVICE: &str = "agent.core";
pub const INIT_SERVICE: &str = "init";
pub const SHELL_SERVICE: &str = "shell";
