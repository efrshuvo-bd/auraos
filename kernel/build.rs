use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let root = manifest_dir.parent().expect("workspace root");
    let guest_dir = root.join("userspace").join("guest");
    let guest_ld = guest_dir.join("user.ld");
    let guest_target = guest_dir.join("target");

    println!("cargo:rerun-if-changed={}", guest_dir.join("src").display());
    println!("cargo:rerun-if-changed={}", guest_ld.display());
    println!("cargo:rerun-if-changed={}", guest_dir.join("Cargo.toml").display());
    println!(
        "cargo:rerun-if-changed={}",
        guest_dir.join(".cargo").join("config.toml").display()
    );

    let linker = find_linker();
    let ld_path = guest_ld.display().to_string().replace('\\', "/");

    let status = Command::new("rustup")
        .current_dir(&guest_dir)
        .env("CARGO_TARGET_DIR", &guest_target)
        // Clear inherited encoded rustflags that may include the kernel linker script.
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_TARGET_AARCH64_UNKNOWN_NONE_RUSTFLAGS")
        .args([
            "run",
            "nightly",
            "cargo",
            "build",
            "-Z",
            "build-std=core",
            "--release",
            "--target",
            "aarch64-unknown-none",
            "--bins",
            "--config",
            &format!(
                "target.aarch64-unknown-none.linker=\"{}\"",
                linker.replace('\\', "/")
            ),
            "--config",
            &format!(
                "target.aarch64-unknown-none.rustflags=[\"-C\",\"link-arg=-T{ld_path}\",\"-C\",\"link-arg=--no-eh-frame-hdr\"]"
            ),
        ])
        .status()
        .expect("failed to spawn rustup/cargo for aura-guest");

    if !status.success() {
        panic!("failed to build aura-guest EL0 binaries");
    }

    let guest_release = guest_target
        .join("aarch64-unknown-none")
        .join("release");
    let init = guest_release.join("guest-init");
    let agent = guest_release.join("guest-agent");
    let shell = guest_release.join("guest-shell");

    for p in [&init, &agent, &shell] {
        if !p.exists() {
            panic!("missing guest ELF: {}", p.display());
        }
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

fn find_linker() -> String {
    let vs = Path::new(
        r"C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\Llvm\bin\ld.lld.exe",
    );
    if vs.exists() {
        return vs.display().to_string();
    }
    "rust-lld".into()
}
