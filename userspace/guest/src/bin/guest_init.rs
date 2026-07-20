#![no_std]
#![no_main]

use aura_guest::agent_ipc::{CH_READY, MSG_READY};
use aura_guest::{exit, ipc_recv_wait, waitpid_noblock, write};

/// Wait for Agent Core READY. Kernel still spawns agent/shell from initrd;
/// init enforces fail-closed policy and exercises SYS_WAITPID (SCRUM-37).
/// Full init-owned spawn of agent/shell is deferred.
const WAIT_YIELDS: u32 = 200_000;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    write("init: AuraOS PID 1 online\n");
    write("init: starting Agent Core (required)\n");
    write("init: waitpid path ready (kernel still spawns guests; init-owned spawn deferred)\n");

    let ready = ipc_recv_wait(CH_READY, WAIT_YIELDS);
    if ready != MSG_READY {
        write("init: FAIL CLOSED — Agent Core did not become ready\n");
        write("init: refusing normal shell session\n");
        exit();
    }

    // If agent already exited, refuse shell (process-based fail-closed).
    if waitpid_noblock(0).is_some() {
        write("init: FAIL CLOSED — peer exited before shell handoff\n");
        write("init: refusing normal shell session\n");
        exit();
    }

    write("init: Agent Core ready\n");
    write("init: starting shell\n");
    exit();
}
