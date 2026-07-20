//! On-device / boot-adjacent OTA verify (SCRUM-41 / SCRUM-49).
//!
//! Pure-Rust SHA-256 + `sha256-dev:` accept/reject, plus soft `ed25519:` via
//! `ed25519-compact` (`default-features = false`, no build script) — **no_std**.
//! Canonical form / pubkey **must stay in sync** with `shared::ota`.
//!
//! Soft ed25519 is **not** HSM-backed and **not** silicon verified boot.
//! Production shipping still requires rotated keys + HSM + boot chain — see
//! `docs/updates-4y.md` and `ota/dev-keys/README.md`.

use ed25519_compact::{PublicKey, Signature};

/// Legacy host token (literal). Kernel prefers `sha256-dev:` / `ed25519:`; kept for parity.
#[allow(dead_code)]
pub const DEV_SIGNATURE: &str = "dev-signed";

pub const SHA256_DEV_PREFIX: &str = "sha256-dev:";

/// Prefix for software ed25519 signatures (`ed25519:<128 hex chars>` = 64 bytes).
pub const ED25519_SOFT_PREFIX: &str = "ed25519:";

/// Dev-only salt (clearly not an HSM secret) — identical to `shared::ota`.
const DEV_DIGEST_SALT: &[u8] = b"AuraOS-ota-dev-salt-v1-NOT-HSM";

/// RFC 8032 test-vector seed public key (dev/QEMU only — not production / not HSM).
/// Must match `shared::ota::DEV_ED25519_PUBLIC_KEY`.
pub const DEV_ED25519_PUBLIC_KEY: [u8; 32] = [
    0xd7, 0x5a, 0x98, 0x01, 0x82, 0xb1, 0x0a, 0xb7, 0xd5, 0x4b, 0xfe, 0xd3, 0xc9, 0x64, 0x07, 0x3a,
    0x0e, 0xe1, 0x72, 0xf3, 0xda, 0xa6, 0x23, 0x25, 0xaf, 0x02, 0x1a, 0x68, 0xf7, 0x07, 0x51, 0x1a,
];

/// Boot-demo fields matching `ota/fixtures/signed-sha256-dev-os.json` /
/// `signed-ed25519-soft-os.json`.
pub const BOOT_DEMO_CHANNEL: &str = "os";
pub const BOOT_DEMO_VERSION: &str = "0.1.1";
pub const BOOT_DEMO_SLOT: &str = "B";
pub const BOOT_DEMO_PAYLOAD_SHA256: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";
/// Expected digest for the boot-demo canonical payload + salt.
pub const BOOT_DEMO_SIG: &str =
    "sha256-dev:fea8b7c20660696b1bdfeb0f23e4c7caa7d5cec5d40d1b5b70dfd28e83094b27";
/// Soft ed25519 over canonical bytes (no salt) — matches host fixture.
pub const BOOT_DEMO_ED25519_SIG: &str = concat!(
    "ed25519:",
    "e6df346c70c22e60038ae2a2e3f28d82975327012db471adf382114b30bc2633",
    "2727ba540437a3c5289ab5d2c6596e1fa6c67e2d8ff19321f42340ac4a25a80b"
);
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VerifyError {
    UnknownChannel,
    Unsigned,
    BadSignature,
}

/// Minimal manifest view (no serde / no alloc).
#[derive(Clone, Copy)]
pub struct ManifestView<'a> {
    pub channel: &'a str,
    pub version: &'a str,
    pub target_slot: &'a str,
    pub payload_sha256: &'a str,
    pub signature: &'a str,
}

impl<'a> ManifestView<'a> {
    pub const fn boot_demo_signed() -> Self {
        Self {
            channel: BOOT_DEMO_CHANNEL,
            version: BOOT_DEMO_VERSION,
            target_slot: BOOT_DEMO_SLOT,
            payload_sha256: BOOT_DEMO_PAYLOAD_SHA256,
            signature: BOOT_DEMO_SIG,
        }
    }

    pub const fn boot_demo_unsigned() -> Self {
        Self {
            channel: BOOT_DEMO_CHANNEL,
            version: BOOT_DEMO_VERSION,
            target_slot: BOOT_DEMO_SLOT,
            payload_sha256: BOOT_DEMO_PAYLOAD_SHA256,
            signature: "",
        }
    }

    pub const fn boot_demo_bad_sig() -> Self {
        Self {
            channel: BOOT_DEMO_CHANNEL,
            version: BOOT_DEMO_VERSION,
            target_slot: BOOT_DEMO_SLOT,
            payload_sha256: BOOT_DEMO_PAYLOAD_SHA256,
            signature: "sha256-dev:0000000000000000000000000000000000000000000000000000000000000000",
        }
    }

    pub const fn boot_demo_ed25519() -> Self {
        Self {
            channel: BOOT_DEMO_CHANNEL,
            version: BOOT_DEMO_VERSION,
            target_slot: BOOT_DEMO_SLOT,
            payload_sha256: BOOT_DEMO_PAYLOAD_SHA256,
            signature: BOOT_DEMO_ED25519_SIG,
        }
    }

    pub const fn boot_demo_ed25519_bad() -> Self {
        Self {
            channel: BOOT_DEMO_CHANNEL,
            version: BOOT_DEMO_VERSION,
            target_slot: BOOT_DEMO_SLOT,
            payload_sha256: BOOT_DEMO_PAYLOAD_SHA256,
            signature: "ed25519:00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        }
    }
}

fn channel_ok(name: &str) -> bool {
    matches!(name, "os" | "agent" | "models")
}

/// Fail-closed verify: unsigned / bad digest / unknown channel → error.
pub fn verify_manifest_view(m: &ManifestView<'_>) -> Result<(), VerifyError> {
    if !channel_ok(m.channel) {
        return Err(VerifyError::UnknownChannel);
    }
    let sig = m.signature.trim();
    if sig.is_empty() {
        return Err(VerifyError::Unsigned);
    }
    if sig == DEV_SIGNATURE {
        // Legacy token accepted for parity with host; boot path uses sha256-dev.
        return Ok(());
    }
    if let Some(got) = sig.strip_prefix(SHA256_DEV_PREFIX) {
        let mut expect = [0u8; 64];
        digest_hex_into(m, &mut expect);
        if eq_ignore_ascii_case(got.as_bytes(), &expect) {
            return Ok(());
        }
        return Err(VerifyError::BadSignature);
    }
    if let Some(hex) = sig.strip_prefix(ED25519_SOFT_PREFIX) {
        let Some(sig_bytes) = hex_decode_64(hex) else {
            return Err(VerifyError::BadSignature);
        };
        let mut msg = [0u8; 256];
        let n = canonical_sign_bytes_into(m, &mut msg);
        if soft_ed25519_verify(&msg[..n], &sig_bytes, &DEV_ED25519_PUBLIC_KEY) {
            return Ok(());
        }
        return Err(VerifyError::BadSignature);
    }
    Err(VerifyError::BadSignature)
}

/// Soft ed25519 verify (on-device). Same crate as host SoftEd25519 — not HSM.
fn soft_ed25519_verify(message: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
    let Ok(pk) = PublicKey::from_slice(public_key) else {
        return false;
    };
    let Ok(sig) = Signature::from_slice(signature) else {
        return false;
    };
    pk.verify(message, &sig).is_ok()
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

/// Canonical bytes for soft ed25519 (no salt) — identical to `shared::ota::canonical_sign_bytes`.
fn canonical_sign_bytes_into(m: &ManifestView<'_>, buf: &mut [u8; 256]) -> usize {
    let mut n = 0usize;
    n = append(buf, n, b"aura-ota-v1\nchannel=");
    n = append(buf, n, m.channel.as_bytes());
    n = append(buf, n, b"\nversion=");
    n = append(buf, n, m.version.as_bytes());
    n = append(buf, n, b"\ntarget_slot=");
    n = append(buf, n, m.target_slot.as_bytes());
    n = append(buf, n, b"\npayload_sha256=");
    n = append(buf, n, m.payload_sha256.as_bytes());
    n = append(buf, n, b"\n");
    n
}

fn digest_hex_into(m: &ManifestView<'_>, out: &mut [u8; 64]) {
    // Canonical form + salt into a fixed stack buffer (no alloc).
    let mut buf = [0u8; 384];
    let mut n = 0usize;
    n = append(&mut buf, n, b"aura-ota-v1\nchannel=");
    n = append(&mut buf, n, m.channel.as_bytes());
    n = append(&mut buf, n, b"\nversion=");
    n = append(&mut buf, n, m.version.as_bytes());
    n = append(&mut buf, n, b"\ntarget_slot=");
    n = append(&mut buf, n, m.target_slot.as_bytes());
    n = append(&mut buf, n, b"\npayload_sha256=");
    n = append(&mut buf, n, m.payload_sha256.as_bytes());
    n = append(&mut buf, n, b"\n");
    n = append(&mut buf, n, DEV_DIGEST_SALT);
    let digest = sha256(&buf[..n]);
    hex_encode(&digest, out);
}
fn append(buf: &mut [u8], n: usize, bytes: &[u8]) -> usize {
    let end = n + bytes.len();
    if end > buf.len() {
        return n; // truncate (boot-demo fields are well under capacity)
    }
    buf[n..end].copy_from_slice(bytes);
    end
}

fn eq_ignore_ascii_case(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (x, y) in a.iter().zip(b.iter()) {
        if x.to_ascii_lowercase() != y.to_ascii_lowercase() {
            return false;
        }
    }
    true
}

fn hex_encode(bytes: &[u8; 32], out: &mut [u8; 64]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for (i, &b) in bytes.iter().enumerate() {
        out[i * 2] = HEX[(b >> 4) as usize];
        out[i * 2 + 1] = HEX[(b & 0xf) as usize];
    }
}

/// Minimal SHA-256 (FIPS 180-4) — stack padding, no `alloc`.
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

    let bit_len = (message.len() as u64).wrapping_mul(8);
    // message || 0x80 || zeros || bit_len — boot-demo canonical+salt is ~162 bytes
    // → padded length 192; keep headroom for slightly larger views.
    let mut padded = [0u8; 256];
    if message.len() > 200 {
        // Refuse oversized inputs rather than silently truncate digests.
        return [0u8; 32];
    }
    padded[..message.len()].copy_from_slice(message);
    padded[message.len()] = 0x80;
    let mut len = message.len() + 1;
    while (len % 64) != 56 {
        len += 1;
    }
    padded[len..len + 8].copy_from_slice(&bit_len.to_be_bytes());
    len += 8;

    for chunk in padded[..len].chunks_exact(64) {
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
