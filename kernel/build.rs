//! Embed prebuilt EL0 guest ELFs. Guests are built by `scripts/build-kernel.ps1`
//! (or manually under `userspace/guest`) before the kernel crate compiles.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let root = manifest_dir.parent().expect("workspace root");
    let guest_dir = root.join("userspace").join("guest");
    let guest_release = guest_dir
        .join("target")
        .join("aarch64-unknown-none")
        .join("release");

    println!("cargo:rerun-if-changed={}", guest_dir.join("src").display());
    println!("cargo:rerun-if-changed={}", guest_dir.join("user.ld").display());
    println!("cargo:rerun-if-changed={}", guest_dir.join("Cargo.toml").display());
    println!(
        "cargo:rerun-if-changed={}",
        guest_dir.join(".cargo").join("config.toml").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        guest_release.join("guest-init").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        guest_release.join("guest-agent").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        guest_release.join("guest-shell").display()
    );

    let init = guest_release.join("guest-init");
    let agent = guest_release.join("guest-agent");
    let shell = guest_release.join("guest-shell");

    for p in [&init, &agent, &shell] {
        if !p.exists() {
            panic!(
                "missing guest ELF: {}\nBuild guests first (scripts/build-kernel.ps1 does this).",
                p.display()
            );
        }
        println!("cargo:rerun-if-changed={}", p.display());
    }

    verify_guest_entry(&init);

    let blobs = format!(
        "pub const GUEST_INIT: &[u8] = include_bytes!(r\"{}\");\n\
         pub const GUEST_AGENT: &[u8] = include_bytes!(r\"{}\");\n\
         pub const GUEST_SHELL: &[u8] = include_bytes!(r\"{}\");\n",
        init.display(),
        agent.display(),
        shell.display()
    );
    fs::write(out_dir.join("guest_blobs.rs"), blobs).expect("write guest_blobs.rs");
}

fn verify_guest_entry(path: &Path) {
    let bytes = fs::read(path).expect("read guest-init");
    if bytes.len() < 32 {
        panic!("guest-init too small");
    }
    let entry = u64::from_le_bytes(bytes[24..32].try_into().unwrap());
    if entry < 0x40_0000 || entry >= 0x80_0000 {
        panic!(
            "guest-init entry 0x{entry:x} looks wrong (expected ~0x400000). Check user.ld linking."
        );
    }
}
