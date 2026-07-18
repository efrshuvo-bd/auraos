#![no_std]
#![no_main]

use aura_guest::{exit, ipc_send, write};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    write("agent.core: privileged service online\n");
    ipc_send(2, 0xA11E);
    write("agent.core: tools ready (system_status/echo/list_services/help)\n");
    exit()
}
