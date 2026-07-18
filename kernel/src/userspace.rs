//! Spawn embedded guest ELFs into real EL0 processes.

use crate::console;
use crate::process;

// Built by kernel/build.rs into OUT_DIR.
include!(concat!(env!("OUT_DIR"), "/guest_blobs.rs"));

pub fn spawn_init() {
    console::println("userspace: loading embedded guest ELFs");
    if !process::spawn("init", GUEST_INIT) {
        console::println("userspace: failed to spawn init");
        return;
    }
    if !process::spawn("agent.core", GUEST_AGENT) {
        console::println("userspace: failed to spawn agent.core");
        return;
    }
    if !process::spawn("shell", GUEST_SHELL) {
        console::println("userspace: failed to spawn shell");
        return;
    }
    console::println("userspace: init/agent/shell ready (EL0)");
}
