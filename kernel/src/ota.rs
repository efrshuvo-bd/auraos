//! On-device OTA A/B skeleton (Sprint 6/7 — SCRUM-31 / SCRUM-36).
//!
//! Logs channel/slot contract and an explicit **apply / slot-switch stub**.
//! Does **not** download, verify with production crypto, write slots, or claim
//! a successful apply. Host-side reject-unsigned remains in
//! `tools/ota-verify` + `shared::ota`.

use crate::console;

/// Channels from the 4-year update contract (`ota/channels.json`).
const CHANNELS: &[&str] = &["os", "agent", "models"];

#[derive(Clone, Copy)]
enum Slot {
    A,
    B,
}

impl Slot {
    fn as_str(self) -> &'static str {
        match self {
            Slot::A => "A",
            Slot::B => "B",
        }
    }

    fn other(self) -> Self {
        match self {
            Slot::A => Slot::B,
            Slot::B => Slot::A,
        }
    }
}

/// Boot-time note: A/B metadata exists in-repo; apply path is stubbed only.
pub fn init() {
    console::print("ota: channels=");
    for (i, ch) in CHANNELS.iter().enumerate() {
        if i > 0 {
            console::print(",");
        }
        console::print(ch);
    }
    console::println(" (A/B metadata in ota/; apply stubbed)");

    // In-memory slot view mirroring `ota/slots.json` / `shared::ota::SlotId`.
    let active = Slot::A;
    let inactive = active.other();
    console::print("ota: apply stub: active=");
    console::print(active.as_str());
    console::print(" inactive=");
    console::print(inactive.as_str());
    console::println(" - would switch A<->B");
    console::println("ota: apply stub: refused unsigned (host aura-ota-verify remains authority)");
    console::println("ota: A/B not applied (no crypto / no slot write)");
}
