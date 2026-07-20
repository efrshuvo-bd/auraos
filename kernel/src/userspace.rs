//! Spawn EL0 guests from the QEMU initrd (cpio newc).
//!
//! Sprint 8 / SCRUM-39: kernel boots **init** only; init owns spawn of
//! agent/shell via `SYS_SPAWN` (initrd lookup by role).

use crate::bootinfo;
use crate::console;
use crate::cpio;
use crate::process;

const INIT_FILE: &str = "guest-init";
const AGENT_FILE: &str = "guest-agent";
const SHELL_FILE: &str = "guest-shell";

/// Role ids for `SYS_SPAWN` (must match guest `SPAWN_*` constants).
pub const SPAWN_AGENT: u64 = 1;
pub const SPAWN_SHELL: u64 = 2;

/// Boot path: load only PID 1 (`init`) from initrd.
pub fn spawn_init() {
    console::println("userspace: loading init from initrd (init-owned spawn)");
    let Some(archive) = bootinfo::initrd_slice() else {
        console::println("userspace: no initrd (missing FDT /chosen linux,initrd-*)");
        return;
    };

    let Some(image) = cpio::lookup(archive, INIT_FILE) else {
        console::print("userspace: missing initrd file: ");
        console::println(INIT_FILE);
        return;
    };
    if !process::spawn("init", image) {
        console::println("userspace: failed to spawn init");
        return;
    }
    console::println("userspace: init ready (EL0); agent/shell via SYS_SPAWN");
}

/// Init-only: spawn agent or shell from the same initrd archive.
///
/// Returns the new pid on success, or `-1` on failure / privilege denial.
pub fn spawn_from_initrd(role: u64) -> i64 {
    if process::current_name() != "init" {
        console::println("userspace: SYS_SPAWN denied (not init)");
        return -1;
    }
    let (proc_name, file_name) = match role {
        SPAWN_AGENT => ("agent.core", AGENT_FILE),
        SPAWN_SHELL => ("shell", SHELL_FILE),
        _ => {
            console::println("userspace: SYS_SPAWN bad role");
            return -1;
        }
    };
    let Some(archive) = bootinfo::initrd_slice() else {
        console::println("userspace: SYS_SPAWN no initrd");
        return -1;
    };
    let Some(image) = cpio::lookup(archive, file_name) else {
        console::print("userspace: SYS_SPAWN missing ");
        console::println(file_name);
        return -1;
    };
    match process::spawn_returning_pid(proc_name, image) {
        Some(pid) => {
            console::print("userspace: init spawned ");
            console::print(proc_name);
            console::print(" pid=");
            print_u32(pid);
            console::println("");
            pid as i64
        }
        None => {
            console::print("userspace: SYS_SPAWN failed ");
            console::println(proc_name);
            -1
        }
    }
}

fn print_u32(mut v: u32) {
    if v == 0 {
        crate::uart::write_bytes(b"0");
        return;
    }
    let mut buf = [0u8; 10];
    let mut i = 10;
    while v > 0 {
        i -= 1;
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
    }
    crate::uart::write_bytes(&buf[i..]);
}
