#![no_std]
#![no_main]

use aura_guest::agent_ipc::{
    resp_err, resp_ok, tool_name, CH_READY, CH_REQ, CH_RESP, ERR_UNKNOWN_TOOL, MSG_READY,
    MSG_SHUTDOWN, TOOL_ECHO, TOOL_HELP, TOOL_LIST_SERVICES, TOOL_SYSTEM_STATUS,
};
use aura_guest::{exit, ipc_recv, ipc_send, write, yield_now};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    write("agent.core: privileged service online\n");
    ipc_send(CH_READY, MSG_READY);
    write("agent.core: tools ready (help/system_status/list_services/echo)\n");
    write("agent.core: EL0 tool loop running\n");

    loop {
        let req = ipc_recv(CH_REQ);
        if req == 0 {
            yield_now();
            continue;
        }
        // Clear request slot so shell can post the next one.
        ipc_send(CH_REQ, 0);

        if req == MSG_SHUTDOWN {
            write("agent.core: shutdown requested\n");
            exit();
        }

        let resp = match req {
            TOOL_HELP => {
                write("agent.core: tool help\n");
                write("agent.core: tools: help, system_status, list_services, echo\n");
                resp_ok(TOOL_HELP)
            }
            TOOL_SYSTEM_STATUS => {
                write("agent.core: tool system_status\n");
                write("agent.core: status ok (guest EL0; kernel IPC)\n");
                resp_ok(TOOL_SYSTEM_STATUS)
            }
            TOOL_LIST_SERVICES => {
                write("agent.core: tool list_services\n");
                write("agent.core: services: init, agent.core, shell\n");
                resp_ok(TOOL_LIST_SERVICES)
            }
            TOOL_ECHO => {
                write("agent.core: tool echo\n");
                write("agent.core: echo ok\n");
                resp_ok(TOOL_ECHO)
            }
            other => {
                write("agent.core: unknown tool id\n");
                let _ = tool_name(other);
                resp_err(ERR_UNKNOWN_TOOL)
            }
        };
        ipc_send(CH_RESP, resp);
    }
}
