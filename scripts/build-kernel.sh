#!/usr/bin/env bash
# Build AuraOS kernel + guests for QEMU aarch64 virt (Linux/CI).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export PATH="${HOME}/.cargo/bin:${PATH}"

GUEST_DIR="$ROOT/userspace/guest"
GUEST_TARGET="$GUEST_DIR/target"
BUILD_DIR="$ROOT/build"

echo "Building aura-guest EL0 binaries..."
(
  cd "$GUEST_DIR"
  export CARGO_TARGET_DIR="$GUEST_TARGET"
  unset CARGO_ENCODED_RUSTFLAGS RUSTFLAGS CARGO_TARGET_AARCH64_UNKNOWN_NONE_RUSTFLAGS || true
  rustup run nightly cargo build -Z build-std=core --release --target aarch64-unknown-none --bins
)

for bin in guest-init guest-agent guest-shell; do
  p="$GUEST_TARGET/aarch64-unknown-none/release/$bin"
  if [[ ! -f "$p" ]]; then
    echo "Missing guest ELF: $p" >&2
    exit 1
  fi
done

echo "Packing initrd (cpio newc)..."
bash "$ROOT/scripts/pack-initrd.sh"

echo "Building aura-kernel (aarch64-unknown-none)..."
(
  cd "$ROOT/kernel"
  export CARGO_TARGET_DIR="$ROOT/kernel/target"
  cargo +nightly build -Z build-std=core,alloc --target aarch64-unknown-none --release
)

ELF="$ROOT/kernel/target/aarch64-unknown-none/release/aura-kernel"
if [[ ! -f "$ELF" ]]; then
  echo "Kernel ELF not found at $ELF" >&2
  exit 1
fi

mkdir -p "$BUILD_DIR"
BIN="$BUILD_DIR/aura-kernel.bin"
if command -v llvm-objcopy >/dev/null 2>&1; then
  llvm-objcopy -O binary "$ELF" "$BIN"
elif command -v rust-objcopy >/dev/null 2>&1; then
  rust-objcopy -O binary "$ELF" "$BIN"
else
  # llvm-tools-preview provides rust-objcopy via rustup
  OBJCOPY="$(rustup which llvm-objcopy 2>/dev/null || true)"
  if [[ -z "${OBJCOPY}" ]]; then
    OBJCOPY="$(find "$(rustc +nightly --print sysroot)/lib/rustlib" -name 'llvm-objcopy' 2>/dev/null | head -n1 || true)"
  fi
  if [[ -n "${OBJCOPY}" && -x "${OBJCOPY}" ]]; then
    "$OBJCOPY" -O binary "$ELF" "$BIN"
  else
    echo "warning: llvm-objcopy not found; skipping aura-kernel.bin" >&2
  fi
fi

INITRD="$BUILD_DIR/initrd.cpio"
echo "OK: $ELF"
[[ -f "$BIN" ]] && echo "OK: $BIN"
echo "OK: $INITRD"
