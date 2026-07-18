#![no_std]
#![no_main]

use aura_guest::{exit, write};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    write("init: AuraOS PID 1 online\n");
    write("init: starting Agent Core (required)\n");
    write("init: starting shell\n");
    exit()
}
