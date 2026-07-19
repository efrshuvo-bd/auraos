//! Host-side OTA manifest verify stub (Sprint 6 / SCRUM-31).
//!
//! Rejects unsigned payloads. Dev signature contract is the literal string
//! `dev-signed` — not production cryptography. See `ota/dev-keys/README.md`.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

const DEV_SIGNATURE: &str = "dev-signed";
const KNOWN_CHANNELS: &[&str] = &["os", "agent", "models"];

#[derive(Debug, Deserialize)]
struct UpdateManifest {
    channel: String,
    version: String,
    #[serde(default)]
    target_slot: Option<String>,
    #[serde(default)]
    payload_sha256: Option<String>,
    /// Absent, null, or empty → unsigned (must reject).
    #[serde(default)]
    signature: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
enum VerifyError {
    UnknownChannel(String),
    Unsigned,
    BadDevSignature,
}

fn verify_manifest(m: &UpdateManifest) -> Result<(), VerifyError> {
    if !KNOWN_CHANNELS.contains(&m.channel.as_str()) {
        return Err(VerifyError::UnknownChannel(m.channel.clone()));
    }
    match m.signature.as_deref().map(str::trim) {
        None | Some("") => Err(VerifyError::Unsigned),
        Some(DEV_SIGNATURE) => Ok(()),
        Some(_) => Err(VerifyError::BadDevSignature),
    }
}

fn load_manifest(path: &Path) -> Result<UpdateManifest> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: aura-ota-verify <manifest.json>");
        return ExitCode::from(2);
    };

    let manifest = match load_manifest(Path::new(&path)) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: {e:#}");
            return ExitCode::FAILURE;
        }
    };

    match verify_manifest(&manifest) {
        Ok(()) => {
            println!(
                "ok: channel={} version={} slot={} (dev signature accepted)",
                manifest.channel,
                manifest.version,
                manifest.target_slot.as_deref().unwrap_or("-")
            );
            let _ = manifest.payload_sha256;
            ExitCode::SUCCESS
        }
        Err(VerifyError::Unsigned) => {
            eprintln!("reject: unsigned OTA payload (missing signature)");
            ExitCode::FAILURE
        }
        Err(VerifyError::BadDevSignature) => {
            eprintln!("reject: signature present but not valid for dev trust anchor");
            ExitCode::FAILURE
        }
        Err(VerifyError::UnknownChannel(ch)) => {
            eprintln!("reject: unknown channel {ch:?} (expected os|agent|models)");
            ExitCode::FAILURE
        }
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
    fn rejects_missing_signature() {
        assert_eq!(verify_manifest(&m("os", None)), Err(VerifyError::Unsigned));
    }

    #[test]
    fn rejects_empty_signature() {
        assert_eq!(
            verify_manifest(&m("agent", Some(""))),
            Err(VerifyError::Unsigned)
        );
    }

    #[test]
    fn accepts_dev_signed() {
        assert_eq!(verify_manifest(&m("models", Some(DEV_SIGNATURE))), Ok(()));
    }

    #[test]
    fn rejects_unknown_channel() {
        assert_eq!(
            verify_manifest(&m("firmware", Some(DEV_SIGNATURE))),
            Err(VerifyError::UnknownChannel("firmware".into()))
        );
    }

    #[test]
    fn rejects_wrong_dev_token() {
        assert_eq!(
            verify_manifest(&m("os", Some("not-a-real-sig"))),
            Err(VerifyError::BadDevSignature)
        );
    }
}
