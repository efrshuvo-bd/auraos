#![no_std]
#![no_main]

use aura_guest::agent_ipc::{
    is_resp_ok, CH_READY, CH_REQ, CH_RESP, MSG_READY, MSG_SHUTDOWN, TOOL_HELP, TOOL_SYSTEM_STATUS,
};
use aura_guest::{exit, ipc_recv_wait, ipc_send, write};

/// Generous yield budget so agent (same RR scheduler) can post READY / replies.
const WAIT_YIELDS: u32 = 200_000;

fn request_tool(tool_id: u64, label: &str) -> bool {
    ipc_send(CH_RESP, 0);
    ipc_send(CH_REQ, tool_id);
    let resp = ipc_recv_wait(CH_RESP, WAIT_YIELDS);
    if resp == 0 {
        write("shell: tool timeout — Agent Core not responding\n");
        return false;
    }
    if is_resp_ok(resp) && (resp & 0xFFFF) == tool_id {
        write(label);
        true
    } else {
        write("shell: tool error response\n");
        false
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    write("shell: home + agent overlay ready\n");

    let ready = ipc_recv_wait(CH_READY, WAIT_YIELDS);
    if ready != MSG_READY {
        write("shell: FAIL CLOSED — Agent Core not ready\n");
        write("shell: refusing normal session\n");
        exit();
    }
    write("shell: agent handshake ok (IPC READY)\n");

    if !request_tool(TOOL_HELP, "shell: help ok via Agent Core\n") {
        write("shell: FAIL CLOSED — help failed\n");
        exit();
    }
    if !request_tool(
        TOOL_SYSTEM_STATUS,
        "shell: system_status ok via Agent Core\n",
    ) {
        write("shell: FAIL CLOSED — system_status failed\n");
        exit();
    }

    write("syscall: write\n");
    write("shell: spinning for preempt smoke\n");
    for _ in 0..40_000_000u32 {
        core::hint::spin_loop();
    }
    write("shell: spin done\n");

    ipc_send(CH_REQ, MSG_SHUTDOWN);
    write("shell: asked agent to shutdown; rich LLM demos stay on host aura-shell\n");
    exit();
}
