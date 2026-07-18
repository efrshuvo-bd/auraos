//! Embedded EL0 guest ELF images.
//! Built by `scripts/build-kernel.ps1` into `userspace/guest/target/...` before the kernel links.

pub const GUEST_INIT: &[u8] =
    include_bytes!("../../userspace/guest/target/aarch64-unknown-none/release/guest-init");
pub const GUEST_AGENT: &[u8] =
    include_bytes!("../../userspace/guest/target/aarch64-unknown-none/release/guest-agent");
pub const GUEST_SHELL: &[u8] =
    include_bytes!("../../userspace/guest/target/aarch64-unknown-none/release/guest-shell");
