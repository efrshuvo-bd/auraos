//! Host-side OTA manifest verify (Sprint 6 / SCRUM-31 / Sprint 8 SCRUM-41).
//!
//! Rejects unsigned payloads. Accepts legacy `dev-signed` or `sha256-dev:<hex>`
//! against the in-tree **dev** salt (not HSM / not ed25519 yet).
//! See `ota/dev-keys/README.md` and `docs/updates-4y.md`.

use anyhow::{Context, Result};
use shared::ota::{self, UpdateManifest, VerifyError};
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

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

    match ota::verify_manifest(&manifest) {
        Ok(()) => {
            let kind = match manifest.signature.as_deref() {
                Some(s) if s.starts_with(ota::SHA256_DEV_PREFIX) => {
                    "sha256-dev digest (dev salt; not HSM)"
                }
                Some(ota::DEV_SIGNATURE) => "dev-signed token (legacy stub)",
                _ => "accepted",
            };
            println!(
                "ok: channel={} version={} slot={} ({})",
                manifest.channel,
                manifest.version,
                manifest.target_slot.as_deref().unwrap_or("-"),
                kind
            );
            let _ = manifest.payload_sha256;
            ExitCode::SUCCESS
        }
        Err(VerifyError::Unsigned) => {
            eprintln!("reject: unsigned OTA payload (missing signature)");
            ExitCode::FAILURE
        }
        Err(VerifyError::BadSignature) => {
            eprintln!("reject: signature present but not valid for trust anchor");
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
    use shared::ota::{
        sign_manifest_sha256_dev, verify_manifest, DEV_SIGNATURE, VerifyError,
    };
    use std::path::PathBuf;

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

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../ota/fixtures")
            .join(name)
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
            Err(VerifyError::BadSignature)
        );
    }

    #[test]
    fn sha256_dev_round_trip() {
        let mut manifest = m("os", None);
        manifest.signature = Some(sign_manifest_sha256_dev(&manifest));
        assert_eq!(verify_manifest(&manifest), Ok(()));
    }

    #[test]
    fn fixture_rejects_unsigned_os() {
        let manifest = load_manifest(&fixture_path("unsigned-os.json")).expect("load");
        assert_eq!(verify_manifest(&manifest), Err(VerifyError::Unsigned));
    }

    #[test]
    fn fixture_accepts_signed_os() {
        let manifest = load_manifest(&fixture_path("signed-os.json")).expect("load");
        assert_eq!(verify_manifest(&manifest), Ok(()));
    }

    #[test]
    fn fixture_rejects_unsigned_agent() {
        let manifest = load_manifest(&fixture_path("unsigned-agent.json")).expect("load");
        assert_eq!(verify_manifest(&manifest), Err(VerifyError::Unsigned));
    }

    #[test]
    fn fixture_accepts_signed_agent() {
        let manifest = load_manifest(&fixture_path("signed-agent.json")).expect("load");
        assert_eq!(verify_manifest(&manifest), Ok(()));
    }

    #[test]
    fn fixture_rejects_unsigned_models() {
        let manifest = load_manifest(&fixture_path("unsigned-models.json")).expect("load");
        assert_eq!(verify_manifest(&manifest), Err(VerifyError::Unsigned));
    }

    #[test]
    fn fixture_accepts_signed_models() {
        let manifest = load_manifest(&fixture_path("signed-models.json")).expect("load");
        assert_eq!(verify_manifest(&manifest), Ok(()));
    }

    #[test]
    fn fixture_accepts_sha256_dev_os() {
        let manifest = load_manifest(&fixture_path("signed-sha256-dev-os.json")).expect("load");
        assert_eq!(verify_manifest(&manifest), Ok(()));
    }
}
