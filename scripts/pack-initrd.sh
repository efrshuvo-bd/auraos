#!/usr/bin/env bash
# Pack guest ELFs into a cpio newc initrd for QEMU -initrd.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
GUEST_OUT="$ROOT/userspace/guest/target/aarch64-unknown-none/release"
BUILD_DIR="$ROOT/build"
OUT="$BUILD_DIR/initrd.cpio"

mkdir -p "$BUILD_DIR"

files=(guest-init guest-agent guest-shell)
for f in "${files[@]}"; do
  if [[ ! -f "$GUEST_OUT/$f" ]]; then
    echo "Missing guest ELF: $GUEST_OUT/$f - build guests first" >&2
    exit 1
  fi
done

python3 - "$OUT" "$GUEST_OUT" <<'PY'
import os, sys

out_path, guest_out = sys.argv[1], sys.argv[2]
names = ["guest-init", "guest-agent", "guest-shell"]

def align4(n: int) -> int:
    return (n + 3) & ~3

def write_entry(buf: bytearray, name: str, data: bytes, mode: int = 0o100644) -> None:
    name_b = name.encode("ascii") + b"\0"
    namesize = len(name_b)
    filesize = len(data)
    hdr = (
        b"070701"
        + f"{0:08x}".encode()
        + f"{mode:08x}".encode()
        + f"{0:08x}".encode()  # uid
        + f"{0:08x}".encode()  # gid
        + f"{1:08x}".encode()  # nlink
        + f"{0:08x}".encode()  # mtime
        + f"{filesize:08x}".encode()
        + f"{0:08x}".encode()  # devmajor
        + f"{0:08x}".encode()  # devminor
        + f"{0:08x}".encode()  # rdevmajor
        + f"{0:08x}".encode()  # rdevminor
        + f"{namesize:08x}".encode()
        + f"{0:08x}".encode()  # check
    )
    assert len(hdr) == 110
    buf.extend(hdr)
    buf.extend(name_b)
    buf.extend(b"\0" * (align4(len(hdr) + namesize) - (len(hdr) + namesize)))
    buf.extend(data)
    buf.extend(b"\0" * (align4(filesize) - filesize))

buf = bytearray()
for name in names:
    path = os.path.join(guest_out, name)
    with open(path, "rb") as f:
        write_entry(buf, name, f.read())
write_entry(buf, "TRAILER!!!", b"", mode=0)

with open(out_path, "wb") as f:
    f.write(buf)
print(f"OK: {out_path} ({len(buf)} bytes)")
PY
