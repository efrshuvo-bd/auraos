#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/kernel"
cargo +nightly build -Z build-std=core,alloc --target aarch64-unknown-none --release
echo "OK: $ROOT/kernel/target/aarch64-unknown-none/release/aura-kernel"
