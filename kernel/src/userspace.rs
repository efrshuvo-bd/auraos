//! Spawn EL0 guests from the QEMU initrd (cpio newc).

use crate::bootinfo;
use crate::console;
use crate::cpio;
use crate::process;

const GUESTS: &[(&str, &str)] = &[
    ("init", "guest-init"),
    ("agent.core", "guest-agent"),
    ("shell", "guest-shell"),
];

pub fn spawn_init() {
    console::println("userspace: loading guests from initrd");
    let Some(archive) = bootinfo::initrd_slice() else {
        console::println("userspace: no initrd (missing FDT /chosen linux,initrd-*)");
        return;
    };

    for &(proc_name, file_name) in GUESTS {
        let Some(image) = cpio::lookup(archive, file_name) else {
            console::print("userspace: missing initrd file: ");
            console::println(file_name);
            return;
        };
        if !process::spawn(proc_name, image) {
            console::print("userspace: failed to spawn ");
            console::println(proc_name);
            return;
        }
    }
    console::println("userspace: init/agent/shell ready (EL0)");
}
