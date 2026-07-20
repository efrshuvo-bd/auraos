//! On-device OTA A/B apply (Sprint 6–9 — SCRUM-31 / SCRUM-36 / SCRUM-40 / SCRUM-41 / SCRUM-45).
//!
//! Sprint 8: real inactive-slot write via VirtIO-blk when the device is present,
//! gated by an **on-device** `sha256-dev:` verify path (fail-closed). Host
//! `aura-ota-verify` shares the same digest algorithm in `shared::ota`.
//! Sprint 9: boot-adjacent VB stub (`vb::allow_activate`) must pass before apply.
//! Production HSM / silicon verified boot remain roadmap — see
//! `docs/updates-4y.md` and `docs/verified-boot.md`.

use crate::console;
use crate::ota_crypto::{self, ManifestView, VerifyError};
use crate::vb;
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

/// Boot-time OTA: on-device verify (fail-closed), then (when blk ready) write inactive slot.
pub fn init() {
    console::print("ota: channels=");
    for (i, ch) in CHANNELS.iter().enumerate() {
        if i > 0 {
            console::print(",");
        }
        console::print(ch);
    }
    console::println(" (A/B metadata in ota/; apply path Sprint 8)");

    // Demonstrate fail-closed paths before any slot write.
    match ota_crypto::verify_manifest_view(&ManifestView::boot_demo_unsigned()) {
        Err(VerifyError::Unsigned) => {
            console::println("ota: verify: refused unsigned (fail-closed before slot write)");
        }
        other => {
            console::print("ota: verify: unexpected unsigned result (");
            log_verify(other);
            console::println(")");
            return;
        }
    }
    match ota_crypto::verify_manifest_view(&ManifestView::boot_demo_bad_sig()) {
        Err(VerifyError::BadSignature) => {
            console::println("ota: verify: rejected bad sha256-dev (fail-closed)");
        }
        other => {
            console::print("ota: verify: unexpected bad-sig result (");
            log_verify(other);
            console::println(")");
            return;
        }
    }

    // Accept path: boot-demo manifest matching ota/fixtures/signed-sha256-dev-os.json.
    match ota_crypto::verify_manifest_view(&ManifestView::boot_demo_signed()) {
        Ok(()) => {
            console::println(
                "ota: verify: sha256-dev ok (on-device; not HSM / not VB / not ed25519)",
            );
        }
        Err(e) => {
            console::print("ota: verify: sha256-dev failed (");
            log_verify_err(e);
            console::println(") — A/B not applied");
            return;
        }
    }

    if !virtio::block_ready() {
        console::println("ota: A/B not applied (no virtio-blk for slot write)");
        return;
    }

    if !vb::allow_activate() {
        console::println("ota: A/B not applied (VB stub refused activate)");
        return;
    }

    match apply_inactive_slot_write() {
        Ok((active, inactive)) => {
            console::print("ota: apply real: wrote inactive=");
            console::print(inactive.as_str());
            console::print(" flipped active=");
            console::print(active.as_str());
            console::println(" (virtio-blk)");
            console::println("ota: A/B slot write ok (unsigned/bad-sig still refused above)");
        }
        Err(msg) => {
            console::print("ota: A/B write failed - ");
            console::println(msg);
        }
    }
}

fn log_verify(r: Result<(), VerifyError>) {
    match r {
        Ok(()) => console::print("ok"),
        Err(e) => log_verify_err(e),
    }
}

fn log_verify_err(e: VerifyError) {
    match e {
        VerifyError::Unsigned => console::print("unsigned"),
        VerifyError::BadSignature => console::print("bad-sig"),
        VerifyError::UnknownChannel => console::print("unknown-channel"),
    }
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
