//! On-device OTA A/B skeleton (Sprint 6 / SCRUM-31).
//!
//! Logs channel/slot contract only. Does **not** download, verify, write slots,
//! or claim a successful apply. Host-side reject-unsigned lives in
//! `tools/ota-verify` + `shared::ota`.

use crate::console;

/// Channels from the 4-year update contract (`ota/channels.json`).
const CHANNELS: &[&str] = &["os", "agent", "models"];

/// Boot-time note: A/B metadata exists in-repo; apply path is not wired.
pub fn init() {
    console::print("ota: channels=");
    for (i, ch) in CHANNELS.iter().enumerate() {
        if i > 0 {
            console::print(",");
        }
        console::print(ch);
    }
    console::println(" (A/B metadata in ota/; apply deferred)");
    console::println("ota: A/B not applied");
}
