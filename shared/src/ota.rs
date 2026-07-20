//! OTA A/B manifest types shared by host verify and future on-device agents.
//!
//! Sprint 6 / SCRUM-31: channels + slot metadata only. Production cryptography
//! is intentionally **not** implemented here — see `ota/dev-keys/README.md`.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Dev-only signature token accepted by the host verify stub.
pub const DEV_SIGNATURE: &str = "dev-signed";

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

/// A/B scheme metadata (host / docs contract; not applied on-device yet).
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
    /// Absent, null, or empty → unsigned (must reject).
    #[serde(default)]
    pub signature: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum VerifyError {
    #[error("unknown channel {0:?} (expected os|agent|models)")]
    UnknownChannel(String),
    #[error("unsigned OTA payload (missing signature)")]
    Unsigned,
    #[error("signature present but not valid for dev trust anchor")]
    BadDevSignature,
}

/// Host / design verify: reject unknown channels and anything not `dev-signed`.
///
/// Production must replace this with real crypto under verified boot.
pub fn verify_manifest(m: &UpdateManifest) -> Result<(), VerifyError> {
    if Channel::parse(&m.channel).is_none() {
        return Err(VerifyError::UnknownChannel(m.channel.clone()));
    }
    match m.signature.as_deref().map(str::trim) {
        None | Some("") => Err(VerifyError::Unsigned),
        Some(DEV_SIGNATURE) => Ok(()),
        Some(_) => Err(VerifyError::BadDevSignature),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(channel: &str, signature: Option<&str>) -> UpdateManifest {
        UpdateManifest {
            channel: channel.into(),
            version: "0.1.1".into(),
            target_slot: Some("B".into()),
            payload_sha256: None,
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
}
