# AuraOS helper commands (optional; requires `cargo install just`)

default:
    @just --list

build:
    cargo build -p shared -p aura-agent -p aura-init -p aura-shell

test:
    cargo test -p shared

run-shell:
    cargo run -p aura-shell

run-init:
    cargo run -p aura-init

kernel:
    powershell -File scripts/build-kernel.ps1

qemu:
    powershell -File scripts/run-qemu.ps1
