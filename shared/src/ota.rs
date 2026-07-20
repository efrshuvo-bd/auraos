//! OTA A/B manifest types shared by host verify and future on-device agents.
//!
//! Sprint 6–10 verify API:
//! - Reject unsigned / empty signatures (fail-closed).
//! - Accept legacy token `dev-signed` (not cryptography).
//! - Accept production-leaning `sha256-dev:<hex>` over a canonical payload
//!   (real digest check; **dev** salt only — not HSM).
//! - Accept soft `ed25519:<hex>` over the canonical payload (software ed25519
//!   via `ed25519-compact`; **not** HSM-backed). See `shared::trust`.
//!
//! On-device / boot-adjacent verify lives in `kernel/src/ota_crypto.rs` and uses
//! the **same** SHA-256 + salt + canonical form and soft ed25519 pubkey (keep
//! in sync). Pass any `impl TrustBackend` to [`verify_manifest_with`] for an
//! HSM-ready swap. HSM + silicon verified boot remain roadmap in
//! `docs/updates-4y.md`.

use crate::trust::{SoftEd25519, TrustBackend};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Dev-only signature token (literal match — not cryptography).
pub const DEV_SIGNATURE: &str = "dev-signed";

/// Prefix for SHA-256-dev signatures in manifest `signature` field.
pub const SHA256_DEV_PREFIX: &str = "sha256-dev:";

/// Prefix for software ed25519 signatures (`ed25519:<128 hex chars>` = 64 bytes).
pub const ED25519_SOFT_PREFIX: &str = "ed25519:";

/// Dev-only salt mixed into the digest (clearly not an HSM secret).
pub const DEV_DIGEST_SALT: &[u8] = b"AuraOS-ota-dev-salt-v1-NOT-HSM";

/// RFC 8032 test-vector seed public key (dev/QEMU only — not production / not HSM).
/// Seed `9d61b19d…7f60` → pubkey below; private material must not ship as a product secret.
pub const DEV_ED25519_PUBLIC_KEY: [u8; 32] = [
    0xd7, 0x5a, 0x98, 0x01, 0x82, 0xb1, 0x0a, 0xb7, 0xd5, 0x4b, 0xfe, 0xd3, 0xc9, 0x64, 0x07, 0x3a,
    0x0e, 0xe1, 0x72, 0xf3, 0xda, 0xa6, 0x23, 0x25, 0xaf, 0x02, 0x1a, 0x68, 0xf7, 0x07, 0x51, 0x1a,
];

/// Canonical bytes digested for `sha256-dev:` signatures.
pub fn canonical_sign_bytes(m: &UpdateManifest) -> Vec<u8> {
    let sha = m.payload_sha256.as_deref().unwrap_or("");
    let slot = m.target_slot.as_deref().unwrap_or("");
    format!(
        "aura-ota-v1\nchannel={}\nversion={}\ntarget_slot={}\npayload_sha256={}\n",
        m.channel, m.version, slot, sha
    )
    .into_bytes()
}

fn digest_hex(m: &UpdateManifest) -> String {
    let mut data = canonical_sign_bytes(m);
    data.extend_from_slice(DEV_DIGEST_SALT);
    let digest = sha256(&data);
    hex_encode(&digest)
}

/// Create a `sha256-dev:` signature for tests / fixture generation.
pub fn sign_manifest_sha256_dev(m: &UpdateManifest) -> String {
    format!("{SHA256_DEV_PREFIX}{}", digest_hex(m))
}

/// Update streams required by the 4-year support contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Os,
    Agent,
    /// Optional on-device packs (Tier B); still must be signed when present.
    Models,
}

impl Channel {
    pub const ALL: [Channel; 3] = [Channel::Os, Channel::Agent, Channel::Models];

    pub fn as_str(self) -> &'static str {
        match self {
            Channel::Os => "os",
            Channel::Agent => "agent",
            Channel::Models => "models",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "os" => Some(Channel::Os),
            "agent" => Some(Channel::Agent),
            "models" => Some(Channel::Models),
            _ => None,
        }
    }
}

/// A/B slot identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SlotId {
    A,
    B,
}

impl SlotId {
    pub fn as_str(self) -> &'static str {
        match self {
            SlotId::A => "A",
            SlotId::B => "B",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "A" | "a" => Some(SlotId::A),
            "B" | "b" => Some(SlotId::B),
            _ => None,
        }
    }

    pub fn other(self) -> Self {
        match self {
            SlotId::A => SlotId::B,
            SlotId::B => SlotId::A,
        }
    }
}

/// Per-slot state mirroring `ota/slots.json` semantics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotState {
    pub version: Option<String>,
    pub bootable: bool,
    pub successful_boot: bool,
}

/// A/B scheme metadata (host / docs contract).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbSlots {
    pub scheme: String,
    pub active: SlotId,
    pub slots: AbSlotMap,
    pub rollback_on_failure: bool,
    pub verified_boot: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbSlotMap {
    pub a: SlotState,
    pub b: SlotState,
}

/// Update manifest verified by `aura-ota-verify`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateManifest {
    pub channel: String,
    pub version: String,
    #[serde(default)]
    pub target_slot: Option<String>,
    #[serde(default)]
    pub payload_sha256: Option<String>,
    /// Absent/empty → unsigned. `dev-signed` → legacy. `sha256-dev:<hex>` → digest.
    /// `ed25519:<hex>` → soft software ed25519 (not HSM).
    #[serde(default)]
    pub signature: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum VerifyError {
    #[error("unknown channel {0:?} (expected os|agent|models)")]
    UnknownChannel(String),
    #[error("unsigned OTA payload (missing signature)")]
    Unsigned,
    #[error("signature present but not valid for trust anchor")]
    BadSignature,
}

/// Result of the on-device apply **planning** helper.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyStubAction {
    RefuseUnsigned,
    WouldApply,
}

/// In-memory A/B apply plan (SCRUM-36/40).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApplyStubPlan {
    pub active: SlotId,
    pub inactive: SlotId,
    pub action: ApplyStubAction,
}

pub fn plan_apply_stub(active: SlotId, signed: bool) -> ApplyStubPlan {
    ApplyStubPlan {
        active,
        inactive: active.other(),
        action: if signed {
            ApplyStubAction::WouldApply
        } else {
            ApplyStubAction::RefuseUnsigned
        },
    }
}

/// Host verify: reject unknown channels / unsigned; accept `dev-signed`,
/// matching `sha256-dev:` digests, or soft `ed25519:` (dev pubkey; not HSM).
pub fn verify_manifest(m: &UpdateManifest) -> Result<(), VerifyError> {
    verify_manifest_with(m, &SoftEd25519)
}

/// Same as [`verify_manifest`] but with an explicit [`TrustBackend`] (HSM-ready shape).
pub fn verify_manifest_with(
    m: &UpdateManifest,
    backend: &impl TrustBackend,
) -> Result<(), VerifyError> {
    if Channel::parse(&m.channel).is_none() {
        return Err(VerifyError::UnknownChannel(m.channel.clone()));
    }
    match m.signature.as_deref().map(str::trim) {
        None | Some("") => Err(VerifyError::Unsigned),
        Some(DEV_SIGNATURE) => Ok(()),
        Some(sig) if sig.starts_with(SHA256_DEV_PREFIX) => {
            let got = sig.strip_prefix(SHA256_DEV_PREFIX).unwrap_or("");
            if got.eq_ignore_ascii_case(&digest_hex(m)) {
                Ok(())
            } else {
                Err(VerifyError::BadSignature)
            }
        }
        Some(sig) if sig.starts_with(ED25519_SOFT_PREFIX) => {
            let hex = sig.strip_prefix(ED25519_SOFT_PREFIX).unwrap_or("");
            let Some(sig_bytes) = hex_decode_64(hex) else {
                return Err(VerifyError::BadSignature);
            };
            let msg = canonical_sign_bytes(m);
            if backend.verify_detached(&msg, &sig_bytes, &DEV_ED25519_PUBLIC_KEY) {
                Ok(())
            } else {
                Err(VerifyError::BadSignature)
            }
        }
        Some(_) => Err(VerifyError::BadSignature),
    }
}

fn hex_decode_64(hex: &str) -> Option<[u8; 64]> {
    if hex.len() != 128 {
        return None;
    }
    let mut out = [0u8; 64];
    for i in 0..64 {
        let b = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
        out[i] = b;
    }
    Some(out)
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0xf) as usize] as char);
    }
    out
}

/// Minimal SHA-256 (FIPS 180-4) — no external crypto crates (WDAC-friendly).
/// Also mirrored in `kernel/src/ota_crypto.rs` for on-device verify.
pub fn sha256(message: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let bit_len = (message.len() as u64) * 8;
    let mut msg = message.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (i, v) in h.iter().enumerate() {
        out[i * 4..(i + 1) * 4].copy_from_slice(&v.to_be_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(channel: &str, signature: Option<&str>) -> UpdateManifest {
        UpdateManifest {
            channel: channel.into(),
            version: "0.1.1".into(),
            target_slot: Some("B".into()),
            payload_sha256: Some(
                "0000000000000000000000000000000000000000000000000000000000000000".into(),
            ),
            signature: signature.map(str::to_string),
        }
    }

    #[test]
    fn rejects_unsigned() {
        assert_eq!(verify_manifest(&m("os", None)), Err(VerifyError::Unsigned));
        assert_eq!(
            verify_manifest(&m("agent", Some(""))),
            Err(VerifyError::Unsigned)
        );
    }

    #[test]
    fn accepts_all_channels_when_dev_signed() {
        for ch in Channel::ALL {
            assert_eq!(
                verify_manifest(&m(ch.as_str(), Some(DEV_SIGNATURE))),
                Ok(())
            );
        }
    }

    #[test]
    fn rejects_unknown_channel() {
        assert_eq!(
            verify_manifest(&m("firmware", Some(DEV_SIGNATURE))),
            Err(VerifyError::UnknownChannel("firmware".into()))
        );
    }

    #[test]
    fn slot_other_flips() {
        assert_eq!(SlotId::A.other(), SlotId::B);
        assert_eq!(SlotId::B.other(), SlotId::A);
    }

    #[test]
    fn apply_stub_refuses_unsigned() {
        let p = plan_apply_stub(SlotId::A, false);
        assert_eq!(p.inactive, SlotId::B);
        assert_eq!(p.action, ApplyStubAction::RefuseUnsigned);
    }

    #[test]
    fn apply_stub_would_apply_when_signed() {
        let p = plan_apply_stub(SlotId::B, true);
        assert_eq!(p.inactive, SlotId::A);
        assert_eq!(p.action, ApplyStubAction::WouldApply);
    }

    #[test]
    fn sha256_dev_accept_and_reject() {
        let mut good = m("os", None);
        let sig = sign_manifest_sha256_dev(&good);
        good.signature = Some(sig);
        assert_eq!(verify_manifest(&good), Ok(()));

        let mut bad = good.clone();
        bad.version = "9.9.9".into();
        assert_eq!(verify_manifest(&bad), Err(VerifyError::BadSignature));
    }

    #[test]
    fn rejects_wrong_token() {
        assert_eq!(
            verify_manifest(&m("os", Some("not-a-real-sig"))),
            Err(VerifyError::BadSignature)
        );
    }

    #[test]
    fn sha256_empty_known_vector() {
        // FIPS empty-string digest
        assert_eq!(
            hex_encode(&sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn ed25519_soft_accept_and_reject() {
        // Fixture signature over canonical_sign_bytes for boot-demo fields
        // (signed offline with RFC8032 seed; see ota/fixtures/signed-ed25519-soft-os.json).
        let mut good = m("os", None);
        good.signature = Some(format!(
            "{ED25519_SOFT_PREFIX}e6df346c70c22e60038ae2a2e3f28d82975327012db471adf382114b30bc26332727ba540437a3c5289ab5d2c6596e1fa6c67e2d8ff19321f42340ac4a25a80b"
        ));
        assert_eq!(verify_manifest(&good), Ok(()));

        let mut bad = good.clone();
        bad.version = "9.9.9".into();
        assert_eq!(verify_manifest(&bad), Err(VerifyError::BadSignature));

        use crate::trust::HsmDeferred;
        assert_eq!(
            verify_manifest_with(&good, &HsmDeferred),
            Err(VerifyError::BadSignature)
        );
    }
}
