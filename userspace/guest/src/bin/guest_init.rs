#![no_std]
#![no_main]

use aura_guest::agent_ipc::{CH_READY, MSG_READY};
use aura_guest::{
    exit, ipc_recv_wait, spawn, waitpid_noblock, write, SPAWN_AGENT, SPAWN_SHELL, WAIT_ANY,
};

/// Init owns agent/shell spawn (SCRUM-39). Fail-closed without Agent Core READY.
const WAIT_YIELDS: u32 = 200_000;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    write("init: AuraOS PID 1 online\n");
    write("init: owning spawn of Agent Core + shell\n");
    write("init: waitpid richer (status+pid; wait-any -1; blocking helper)\n");

    let Some(agent_pid) = spawn(SPAWN_AGENT) else {
        write("init: FAIL CLOSED — could not spawn Agent Core\n");
        exit();
    };
    write("init: spawned agent.core\n");
    let _ = agent_pid;

    let ready = ipc_recv_wait(CH_READY, WAIT_YIELDS);
    if ready != MSG_READY {
        write("init: FAIL CLOSED — Agent Core did not become ready\n");
        write("init: refusing normal shell session\n");
        exit();
    }

    // If agent already exited, refuse shell (process-based fail-closed).
    // wait-any via WAIT_ANY (-1) exercises SCRUM-39 AC.
    if let Some((pid, status)) = waitpid_noblock(WAIT_ANY) {
        write("init: FAIL CLOSED — peer exited before shell handoff\n");
        let _ = (pid, status);
        write("init: refusing normal shell session\n");
        exit();
    }

    write("init: Agent Core ready\n");
    write("init: starting shell\n");

    if spawn(SPAWN_SHELL).is_none() {
        write("init: FAIL CLOSED — could not spawn shell\n");
        exit();
    }
    write("init: spawned shell (init-owned lifecycle)\n");
    exit();
}
