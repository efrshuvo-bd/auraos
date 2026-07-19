//! Guest EL0 Agent Core IPC over in-kernel u64 mailboxes.
//!
//! Keep tool IDs / names aligned with `shared::tools` (host JSON path).
//! Host still uses length-prefixed JSON on TCP; guest uses these opcodes.

#![allow(dead_code)]

/// Channel: agent posts READY once after coming online.
pub const CH_READY: u64 = 2;
/// Channel: shell → agent tool request (tool id); agent clears to 0 after take.
pub const CH_REQ: u64 = 3;
/// Channel: agent → shell tool response.
pub const CH_RESP: u64 = 4;

/// Agent is up and accepting tool requests.
pub const MSG_READY: u64 = 0xA11E;
/// Shell asks agent to exit so the scheduler can reach idle (demo teardown).
pub const MSG_SHUTDOWN: u64 = 0xDEAD;

pub const TOOL_HELP: u64 = 1;
pub const TOOL_SYSTEM_STATUS: u64 = 2;
pub const TOOL_LIST_SERVICES: u64 = 3;
pub const TOOL_ECHO: u64 = 4;

/// Response: high 16 bits = OK marker, low 16 = tool id.
pub const RESP_OK_MARK: u64 = 0x4F4B_0000;
/// Response: high 16 bits = ER marker, low 16 = error code.
pub const RESP_ERR_MARK: u64 = 0x4552_0000;

pub const ERR_UNKNOWN_TOOL: u64 = 1;
pub const ERR_TIMEOUT: u64 = 2;

pub fn resp_ok(tool_id: u64) -> u64 {
    RESP_OK_MARK | (tool_id & 0xFFFF)
}

pub fn resp_err(code: u64) -> u64 {
    RESP_ERR_MARK | (code & 0xFFFF)
}

pub fn is_resp_ok(v: u64) -> bool {
    (v & 0xFFFF_0000) == RESP_OK_MARK
}

/// Names aligned with `shared::tools::BUILTIN_TOOLS`.
pub const TOOL_NAMES: &[(u64, &str)] = &[
    (TOOL_HELP, "help"),
    (TOOL_SYSTEM_STATUS, "system_status"),
    (TOOL_LIST_SERVICES, "list_services"),
    (TOOL_ECHO, "echo"),
];

pub fn tool_name(id: u64) -> Option<&'static str> {
    TOOL_NAMES.iter().find(|(i, _)| *i == id).map(|(_, n)| *n)
}
