//! OTA / boot trust backends (Sprint 9–10 / SCRUM-44 / SCRUM-50).
//!
//! Software ed25519 verify first; HSM remains a deferred backend shape so callers
//! can switch without rewriting verify dispatch. Pass any `impl TrustBackend` into
//! `shared::ota::verify_manifest_with` — production swap is a backend swap, not a
//! caller rewrite. This is **not** production key custody — see
//! `docs/updates-4y.md` and `ota/dev-keys/README.md`.

use ed25519_compact::{PublicKey, Signature};

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
}

/// Pluggable verify surface for host (and future on-device) trust checks.
///
/// # Production swap (SCRUM-50)
///
/// 1. Implement `TrustBackend` for a real HSM (PKCS#11 / cloud HSM / TEE).
/// 2. Pass `&YourHsm` into `verify_manifest_with` (or replace `default_host_backend`).
/// 3. Keep `HsmDeferred` for fail-closed demos until that impl exists.
pub trait TrustBackend {
    fn kind(&self) -> TrustBackendKind;

    /// Detached ed25519 verify over `message` with raw 32-byte pubkey + 64-byte sig.
    fn verify_detached(&self, message: &[u8], signature: &[u8], public_key: &[u8]) -> bool;
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
}

/// Default host backend for Sprint 9–10 demos (soft software; not HSM).
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
}
