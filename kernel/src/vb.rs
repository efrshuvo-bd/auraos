//! Verified-boot / boot-adjacent trust stub (Sprint 9 / SCRUM-45).
//!
//! Documents the intended chain (bootloader → kernel → system) and enforces a
//! **stub** refuse path when trust is marked failed. This is not silicon VB and
//! not HSM-backed — see `docs/verified-boot.md` and `docs/updates-4y.md`.

use crate::console;

/// Stub policy: when false, apply/activate must refuse (fail-closed demo).
static mut BOOT_TRUST_OK: bool = true;

/// Mark boot-adjacent trust failed (test/demo hook).
pub fn set_trust_ok(ok: bool) {
    unsafe {
        BOOT_TRUST_OK = ok;
    }
}

pub fn trust_ok() -> bool {
    unsafe { BOOT_TRUST_OK }
}

/// Boot-time stub gate: demonstrate refuse-on-fail, then restore ok for normal apply.
pub fn init_stub_gate() {
    console::println("vb: stub chain bootloader->kernel->system (not silicon VB / not HSM)");

    // Demonstrate fail-closed refuse before any A/B write.
    set_trust_ok(false);
    if !trust_ok() {
        console::println("vb: stub refuse activate (trust failed; fail-closed)");
    }
    set_trust_ok(true);
    console::println("vb: stub trust ok (software path; HSM deferred)");
}

/// Returns false when stub trust is down — callers must not activate.
pub fn allow_activate() -> bool {
    if trust_ok() {
        true
    } else {
        console::println("vb: stub refuse activate (trust failed; fail-closed)");
        false
    }
}
