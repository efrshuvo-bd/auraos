//! Simulated userspace: init launches Agent Core marker + shell stub in-kernel
//! until ELF loading lands. Host demos use the real `aura-init` / `aura-agent`
//! / `aura-shell` crates.

use crate::console;
use crate::ipc;
use crate::sched;
use crate::syscall::{self, SYS_IPC_RECV, SYS_IPC_SEND, SYS_WRITE, SYS_YIELD};

pub fn spawn_init() {
    let _ = sched::spawn("init", init_task);
    let _ = sched::spawn("agent.core", agent_task);
    let _ = sched::spawn("shell", shell_task);
}

fn init_task() {
    console::println("init: AuraOS PID 1 online");
    console::println("init: starting Agent Core (required)");
    console::println("init: starting shell");
    let _ = syscall::dispatch(SYS_YIELD, 0, 0, 0);
}

fn agent_task() {
    console::println("agent.core: privileged service online");
    // Channel 1: shell -> agent; channel 2: agent -> shell
    let _ = syscall::dispatch(SYS_IPC_SEND, 2, 0xA11E, 0);
    console::println("agent.core: tools ready (system_status/echo/list_services/help)");
    let _ = syscall::dispatch(SYS_YIELD, 0, 0, 0);
}

fn shell_task() {
    console::println("shell: home + agent overlay ready");
    let token = syscall::dispatch(SYS_IPC_RECV, 2, 0, 0);
    if token == 0xA11E {
        console::println("shell: agent handshake ok (IPC)");
    } else {
        console::println("shell: agent handshake missing");
    }
    let _ = syscall::dispatch(SYS_IPC_SEND, 1, 0x4845_4C50, 0);
    let _ = ipc::messages();
    let _ = syscall::dispatch(SYS_WRITE, 0, 0, 0);
    console::println("shell: demo complete - ask agent on host via cargo run -p aura-shell");
}
