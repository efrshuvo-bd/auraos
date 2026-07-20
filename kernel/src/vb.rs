//! Verified-boot / boot-adjacent trust stub (Sprint 9–11 / SCRUM-45 / SCRUM-55).
//!
//! Models the intended chain (bootloader → kernel → system) as **staged stub
//! checks** with clear serial. Fail-closed refuse is still demonstrated. This is
//! not silicon VB and not HSM-backed — see `docs/verified-boot.md` and
//! `docs/updates-4y.md`.

use crate::console;

/// Stages in the intended verified-boot chain (stub labels only on QEMU).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VbStage {
    Bootloader,
    Kernel,
    System,
}

impl VbStage {
    pub fn as_str(self) -> &'static str {
        match self {
            VbStage::Bootloader => "bootloader",
            VbStage::Kernel => "kernel",
            VbStage::System => "system",
        }
    }
}

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

/// Log a single stage stub check (always "stub ok" today — no silicon verify).
fn stub_check_stage(stage: VbStage) {
    console::println(match stage {
        VbStage::Bootloader => {
            "vb: stage bootloader stub ok (not silicon VB; signature verify deferred)"
        }
        VbStage::Kernel => "vb: stage kernel stub ok (not silicon VB; signature verify deferred)",
        VbStage::System => "vb: stage system stub ok (not silicon VB; soft path only)",
    });
}

/// Boot-time stub gate: staged chain, fail-closed refuse demo, then soft-path ok.
pub fn init_stub_gate() {
    console::println("vb: stub chain bootloader->kernel->system (not silicon VB / not HSM)");
    console::println("vb: silicon path deferred (board ROM/OTP trust anchors not wired)");

    stub_check_stage(VbStage::Bootloader);
    stub_check_stage(VbStage::Kernel);
    stub_check_stage(VbStage::System);

    // Demonstrate fail-closed refuse before any A/B write.
    set_trust_ok(false);
    if !trust_ok() {
        console::println("vb: stub refuse activate (trust failed; fail-closed)");
    }
    set_trust_ok(true);
    console::println("vb: stub trust ok (software path; HSM deferred; silicon deferred)");
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
