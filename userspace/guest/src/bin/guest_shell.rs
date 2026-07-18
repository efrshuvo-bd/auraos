#![no_std]
#![no_main]

use aura_guest::{exit, ipc_recv, ipc_send, write};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    write("shell: home + agent overlay ready\n");
    let token = ipc_recv(2);
    if token == 0xA11E {
        write("shell: agent handshake ok (IPC)\n");
    } else {
        write("shell: agent handshake missing\n");
    }
    ipc_send(1, 0x4845_4C50);
    write("syscall: write\n");
    write("shell: demo complete — ask agent on host via `cargo run -p aura-shell`\n");
    exit()
}
