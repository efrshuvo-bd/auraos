//! OTA / boot trust backends (Sprint 9–11 / SCRUM-44 / SCRUM-50 / SCRUM-54).
//!
//! Software ed25519 verify first; HSM remains a deferred backend shape so callers
//! can switch without rewriting verify dispatch. Pass any `impl TrustBackend` into
//! `shared::ota::verify_manifest_with` — production swap is a backend swap, not a
//! caller rewrite.
//!
//! Sprint 11 adds **custody scaffolding** ([`KeyHandle`] / [`CustodyKind`]): opaque
//! handles that distinguish soft-dev anchors from future HSM slots. This is **not**
//! production key custody and does **not** claim keys live in an HSM — see
//! `docs/updates-4y.md` and `ota/dev-keys/README.md`.

use ed25519_compact::{PublicKey, Signature};

/// Where a trust anchor is expected to live (labeling only until HSM is wired).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CustodyKind {
    /// In-process / in-tree soft-dev pubkey material (RFC8032 test vector, etc.).
    SoftDev,
    /// Future PKCS#11 / cloud HSM / TEE slot — **not implemented**; no live keys.
    HsmSlotDeferred,
}

impl CustodyKind {
    pub fn as_str(self) -> &'static str {
        match self {
            CustodyKind::SoftDev => "soft-dev",
            CustodyKind::HsmSlotDeferred => "hsm-slot-deferred",
        }
    }

    /// True only for soft-dev material that can verify today.
    pub fn is_live(self) -> bool {
        matches!(self, CustodyKind::SoftDev)
    }
}

/// Opaque key reference for callers that will later bind HSM slots.
///
/// Soft handles carry a raw 32-byte ed25519 pubkey in process memory (dev only).
/// HSM handles carry a non-secret slot id string for a future backend — they never
/// contain private key material and never imply the slot exists in hardware.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyHandle {
    /// Soft-dev trust anchor (32-byte public key). Not HSM custody.
    SoftDev { public_key: [u8; 32] },
    /// Placeholder for a future HSM object / PKCS#11 CKA_ID — verify always fails
    /// until a real [`TrustBackend`] binds this id.
    HsmSlot { slot_id: &'static str },
}

impl KeyHandle {
    pub fn custody(&self) -> CustodyKind {
        match self {
            KeyHandle::SoftDev { .. } => CustodyKind::SoftDev,
            KeyHandle::HsmSlot { .. } => CustodyKind::HsmSlotDeferred,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            KeyHandle::SoftDev { .. } => "key-handle:soft-dev",
            KeyHandle::HsmSlot { .. } => "key-handle:hsm-slot-deferred",
        }
    }

    /// Soft handle from a 32-byte pubkey slice; `None` if length mismatch.
    pub fn soft_dev_from_slice(public_key: &[u8]) -> Option<Self> {
        let mut pk = [0u8; 32];
        if public_key.len() != 32 {
            return None;
        }
        pk.copy_from_slice(public_key);
        Some(KeyHandle::SoftDev { public_key: pk })
    }

    /// Deferred HSM slot label — does **not** open or claim a real HSM object.
    pub fn hsm_slot_deferred(slot_id: &'static str) -> Self {
        KeyHandle::HsmSlot { slot_id }
    }

    /// Public key bytes when custody is soft-dev; `None` for HSM placeholders.
    pub fn soft_public_key(&self) -> Option<&[u8; 32]> {
        match self {
            KeyHandle::SoftDev { public_key } => Some(public_key),
            KeyHandle::HsmSlot { .. } => None,
        }
    }
}

/// Which trust implementation is active.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrustBackendKind {
    /// Pure-software ed25519 (`ed25519-compact`, no build script).
    SoftSoftware,
    /// Placeholder for future HSM verify — not implemented.
    HsmDeferred,
}

impl TrustBackendKind {
    pub fn as_str(self) -> &'static str {
        match self {
            TrustBackendKind::SoftSoftware => "soft-software",
            TrustBackendKind::HsmDeferred => "hsm-deferred",
        }
    }

    /// True only when verify can succeed today (soft path).
    pub fn is_implemented(self) -> bool {
        matches!(self, TrustBackendKind::SoftSoftware)
    }

    /// Production shipping still requires a real HSM backend — soft is never "prod ready".
    pub fn is_production_ready(self) -> bool {
        false
    }

    /// Expected custody labeling for this backend kind.
    pub fn custody_kind(self) -> CustodyKind {
        match self {
            TrustBackendKind::SoftSoftware => CustodyKind::SoftDev,
            TrustBackendKind::HsmDeferred => CustodyKind::HsmSlotDeferred,
        }
    }
}

/// Pluggable verify surface for host (and future on-device) trust checks.
///
/// # Production swap (SCRUM-50 / SCRUM-54)
///
/// 1. Implement `TrustBackend` for a real HSM (PKCS#11 / cloud HSM / TEE).
/// 2. Bind [`KeyHandle::HsmSlot`] ids to real objects inside that backend.
/// 3. Pass `&YourHsm` into `verify_manifest_with` (or replace `default_host_backend`).
/// 4. Keep `HsmDeferred` + `KeyHandle::HsmSlot` for fail-closed demos until that exists.
///
/// Do **not** treat soft-dev pubkeys or deferred slot ids as "keys in HSM".
pub trait TrustBackend {
    fn kind(&self) -> TrustBackendKind;

    /// Detached ed25519 verify over `message` with raw 32-byte pubkey + 64-byte sig.
    fn verify_detached(&self, message: &[u8], signature: &[u8], public_key: &[u8]) -> bool;

    /// Verify using an opaque [`KeyHandle`] (Sprint 11 custody scaffolding).
    ///
    /// Soft-dev handles use the embedded pubkey; HSM-slot handles always fail
    /// closed on [`HsmDeferred`] / until a real HSM backend overrides this.
    fn verify_with_handle(&self, message: &[u8], signature: &[u8], handle: &KeyHandle) -> bool {
        match handle {
            KeyHandle::SoftDev { public_key } => {
                self.verify_detached(message, signature, public_key)
            }
            KeyHandle::HsmSlot { .. } => false,
        }
    }
}

/// Software ed25519 backend (dev/QEMU trust anchors only).
#[derive(Clone, Copy, Debug, Default)]
pub struct SoftEd25519;

impl TrustBackend for SoftEd25519 {
    fn kind(&self) -> TrustBackendKind {
        TrustBackendKind::SoftSoftware
    }

    fn verify_detached(&self, message: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
        let Ok(pk) = PublicKey::from_slice(public_key) else {
            return false;
        };
        let Ok(sig) = Signature::from_slice(signature) else {
            return false;
        };
        pk.verify(message, &sig).is_ok()
    }
}

/// Future HSM path — always fails closed until wired.
#[derive(Clone, Copy, Debug, Default)]
pub struct HsmDeferred;

impl TrustBackend for HsmDeferred {
    fn kind(&self) -> TrustBackendKind {
        TrustBackendKind::HsmDeferred
    }

    fn verify_detached(&self, _message: &[u8], _signature: &[u8], _public_key: &[u8]) -> bool {
        false
    }

    fn verify_with_handle(&self, _message: &[u8], _signature: &[u8], _handle: &KeyHandle) -> bool {
        false
    }
}

/// Default host backend for Sprint 9–11 demos (soft software; not HSM).
pub fn default_host_backend() -> SoftEd25519 {
    SoftEd25519
}

/// Policy helper: prefer HSM when requested, else soft software.
///
/// When `prefer_hsm` is true, returns [`HsmDeferred`] (fail-closed until a real
/// HSM `TrustBackend` exists). Callers that need live verify must keep using soft
/// or supply their own backend — this helper never pretends HSM works.
pub fn select_host_backend_kind(prefer_hsm: bool) -> TrustBackendKind {
    if prefer_hsm {
        TrustBackendKind::HsmDeferred
    } else {
        TrustBackendKind::SoftSoftware
    }
}

/// Soft-dev handle wrapping the in-tree RFC8032 test-vector public key.
///
/// Same bytes as `shared::ota::DEV_ED25519_PUBLIC_KEY` — labeled soft custody only.
pub fn soft_dev_rfc8032_handle() -> KeyHandle {
    KeyHandle::SoftDev {
        public_key: [
            0xd7, 0x5a, 0x98, 0x01, 0x82, 0xb1, 0x0a, 0xb7, 0xd5, 0x4b, 0xfe, 0xd3, 0xc9, 0x64,
            0x07, 0x3a, 0x0e, 0xe1, 0x72, 0xf3, 0xda, 0xa6, 0x23, 0x25, 0xaf, 0x02, 0x1a, 0x68,
            0xf7, 0x07, 0x51, 0x1a,
        ],
    }
}

/// Example deferred HSM slot id for docs/tests — **not** a live HSM object.
pub fn example_hsm_slot_handle() -> KeyHandle {
    KeyHandle::hsm_slot_deferred("auraos-prod-ota-ed25519-v1-NOT-WIRED")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_compact::{KeyPair, Seed};

    #[test]
    fn soft_ed25519_accept_reject() {
        let seed = Seed::new([
            0x9d, 0x61, 0xb1, 0x9d, 0xef, 0xfd, 0x5a, 0x60, 0xba, 0x84, 0x4a, 0xf4, 0x92, 0xec,
            0x2c, 0xc4, 0x44, 0x49, 0xc5, 0x69, 0x7b, 0x32, 0x69, 0x19, 0x70, 0x3b, 0xac, 0x03,
            0x1c, 0xae, 0x7f, 0x60,
        ]);
        let kp = KeyPair::from_seed(seed);
        let msg = b"aura-trust-smoke";
        let sig = kp.sk.sign(msg, None);
        let soft = SoftEd25519;
        assert!(soft.verify_detached(msg, sig.as_ref(), kp.pk.as_ref()));
        assert!(!soft.verify_detached(b"tampered", sig.as_ref(), kp.pk.as_ref()));
        assert!(!HsmDeferred.verify_detached(msg, sig.as_ref(), kp.pk.as_ref()));
        assert_eq!(soft.kind(), TrustBackendKind::SoftSoftware);
        assert_eq!(HsmDeferred.kind(), TrustBackendKind::HsmDeferred);
        assert!(TrustBackendKind::SoftSoftware.is_implemented());
        assert!(!TrustBackendKind::HsmDeferred.is_implemented());
        assert!(!TrustBackendKind::SoftSoftware.is_production_ready());
        assert_eq!(
            select_host_backend_kind(false),
            TrustBackendKind::SoftSoftware
        );
        assert_eq!(select_host_backend_kind(true), TrustBackendKind::HsmDeferred);
        assert_eq!(soft.kind().as_str(), "soft-software");
        assert_eq!(HsmDeferred.kind().as_str(), "hsm-deferred");
    }

    #[test]
    fn key_handle_custody_scaffolding() {
        let seed = Seed::new([
            0x9d, 0x61, 0xb1, 0x9d, 0xef, 0xfd, 0x5a, 0x60, 0xba, 0x84, 0x4a, 0xf4, 0x92, 0xec,
            0x2c, 0xc4, 0x44, 0x49, 0xc5, 0x69, 0x7b, 0x32, 0x69, 0x19, 0x70, 0x3b, 0xac, 0x03,
            0x1c, 0xae, 0x7f, 0x60,
        ]);
        let kp = KeyPair::from_seed(seed);
        let msg = b"aura-custody-smoke";
        let sig = kp.sk.sign(msg, None);

        let soft_h = KeyHandle::soft_dev_from_slice(kp.pk.as_ref()).expect("32-byte pk");
        assert_eq!(soft_h.custody(), CustodyKind::SoftDev);
        assert!(soft_h.custody().is_live());
        assert_eq!(soft_h.label(), "key-handle:soft-dev");
        assert!(SoftEd25519.verify_with_handle(msg, sig.as_ref(), &soft_h));
        assert!(!SoftEd25519.verify_with_handle(b"tampered", sig.as_ref(), &soft_h));

        let hsm_h = example_hsm_slot_handle();
        assert_eq!(hsm_h.custody(), CustodyKind::HsmSlotDeferred);
        assert!(!hsm_h.custody().is_live());
        assert_eq!(hsm_h.label(), "key-handle:hsm-slot-deferred");
        assert!(hsm_h.soft_public_key().is_none());
        assert!(!SoftEd25519.verify_with_handle(msg, sig.as_ref(), &hsm_h));
        assert!(!HsmDeferred.verify_with_handle(msg, sig.as_ref(), &hsm_h));
        assert!(!HsmDeferred.verify_with_handle(msg, sig.as_ref(), &soft_h));

        assert_eq!(
            TrustBackendKind::SoftSoftware.custody_kind(),
            CustodyKind::SoftDev
        );
        assert_eq!(
            TrustBackendKind::HsmDeferred.custody_kind(),
            CustodyKind::HsmSlotDeferred
        );
        assert_eq!(CustodyKind::SoftDev.as_str(), "soft-dev");
        assert_eq!(CustodyKind::HsmSlotDeferred.as_str(), "hsm-slot-deferred");
    }
}
