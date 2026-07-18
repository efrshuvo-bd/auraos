#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ELF="$ROOT/kernel/target/aarch64-unknown-none/release/aura-kernel"
if [[ ! -f "$ELF" ]]; then
  "$ROOT/scripts/build-kernel.sh"
fi
exec qemu-system-aarch64 \
  -machine virt \
  -cpu cortex-a57 \
  -m 512M \
  -nographic \
  -serial mon:stdio \
  -kernel "$ELF"
