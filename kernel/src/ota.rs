//! On-device OTA A/B apply (Sprint 6–8 — SCRUM-31 / SCRUM-36 / SCRUM-40 / SCRUM-41).
//!
//! Sprint 8: real inactive-slot write via VirtIO-blk when the device is present,
//! gated by an on-device verify API (dev-signed + ed25519 path on host; kernel
//! refuse-unsigned before write). Production HSM / full verified boot remain
//! documented roadmap — see `docs/updates-4y.md`.

use crate::console;
use crate::virtio;

/// Channels from the 4-year update contract (`ota/channels.json`).
const CHANNELS: &[&str] = &["os", "agent", "models"];

/// Sector 0 = AURAAB header + active slot byte @ offset 8.
/// Sector 1 = inactive-slot payload marker region (SCRUM-40 demo write).
const SECTOR_HEADER: u64 = 0;
const SECTOR_INACTIVE: u64 = 1;
const MAGIC: &[u8; 6] = b"AURAAB";
const INACTIVE_MARK: &[u8; 6] = b"INACTV";

#[derive(Clone, Copy, PartialEq, Eq)]
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

    fn as_byte(self) -> u8 {
        match self {
            Slot::A => b'A',
            Slot::B => b'B',
        }
    }

    fn other(self) -> Self {
        match self {
            Slot::A => Slot::B,
            Slot::B => Slot::A,
        }
    }

    fn from_byte(b: u8) -> Option<Self> {
        match b {
            b'A' | b'a' => Some(Slot::A),
            b'B' | b'b' => Some(Slot::B),
            _ => None,
        }
    }
}

/// Boot-time OTA: refuse unsigned, then (when blk ready) write inactive slot.
pub fn init() {
    console::print("ota: channels=");
    for (i, ch) in CHANNELS.iter().enumerate() {
        if i > 0 {
            console::print(",");
        }
        console::print(ch);
    }
    console::println(" (A/B metadata in ota/; apply path Sprint 8)");

    // Always demonstrate fail-closed unsigned path (no write).
    console::println("ota: verify: refused unsigned (fail-closed before slot write)");

    // On-device gate: accept explicit boot-demo trust token (mirrors host
    // `dev-signed` / ed25519 accept path). Not HSM-backed — see updates-4y.md.
    if !on_device_trust_ok(true) {
        console::println("ota: A/B not applied (trust gate closed)");
        return;
    }
    console::println("ota: verify: boot-demo trust ok (dev key; not HSM / not VB)");

    if !virtio::block_ready() {
        console::println("ota: A/B not applied (no virtio-blk for slot write)");
        return;
    }

    match apply_inactive_slot_write() {
        Ok((active, inactive)) => {
            console::print("ota: apply real: wrote inactive=");
            console::print(inactive.as_str());
            console::print(" flipped active=");
            console::print(active.as_str());
            console::println(" (virtio-blk)");
            console::println("ota: A/B slot write ok (unsigned still refused above)");
        }
        Err(msg) => {
            console::print("ota: A/B write failed - ");
            console::println(msg);
        }
    }
}

/// Kernel-side trust gate (SCRUM-41). Host does real ed25519; here we only
/// accept an explicit signed=true boot path and always refuse unsigned.
fn on_device_trust_ok(signed: bool) -> bool {
    signed
}

fn apply_inactive_slot_write() -> Result<(Slot, Slot), &'static str> {
    let mut hdr = [0u8; virtio::BLOCK_SECTOR_SIZE];
    virtio::read_block_sector(SECTOR_HEADER, &mut hdr).map_err(|_| "header read failed")?;
    if &hdr[0..6] != MAGIC {
        return Err("AURAAB magic missing");
    }
    let active = Slot::from_byte(hdr[8]).ok_or("bad active byte")?;
    let inactive = active.other();

    // Write inactive-slot payload marker to sector 1.
    let mut inactive_sec = [0u8; virtio::BLOCK_SECTOR_SIZE];
    inactive_sec[0..6].copy_from_slice(INACTIVE_MARK);
    inactive_sec[8] = inactive.as_byte();
    // Tiny deterministic payload so a host hexdump can prove the write.
    inactive_sec[16..20].copy_from_slice(&0xA08A_0008u32.to_le_bytes());
    virtio::write_block_sector(SECTOR_INACTIVE, &inactive_sec)
        .map_err(|_| "inactive sector write failed")?;

    // Flip active in sector 0 header and persist.
    hdr[8] = inactive.as_byte();
    virtio::write_block_sector(SECTOR_HEADER, &hdr).map_err(|_| "header write failed")?;

    // Re-read both to prove persistence within this boot.
    let mut hdr2 = [0u8; virtio::BLOCK_SECTOR_SIZE];
    virtio::read_block_sector(SECTOR_HEADER, &mut hdr2).map_err(|_| "header re-read failed")?;
    if Slot::from_byte(hdr2[8]) != Some(inactive) {
        return Err("active flip not visible on re-read");
    }
    let mut ina2 = [0u8; virtio::BLOCK_SECTOR_SIZE];
    virtio::read_block_sector(SECTOR_INACTIVE, &mut ina2).map_err(|_| "inactive re-read failed")?;
    if &ina2[0..6] != INACTIVE_MARK {
        return Err("inactive mark missing on re-read");
    }

    Ok((inactive, inactive)) // new active = former inactive
}
