//! Minimal Flattened Device Tree reader for `/chosen` initrd props.

const FDT_MAGIC: u32 = 0xd00d_feed;
const FDT_BEGIN_NODE: u32 = 0x1;
const FDT_END_NODE: u32 = 0x2;
const FDT_PROP: u32 = 0x3;
const FDT_NOP: u32 = 0x4;
const FDT_END: u32 = 0x9;

fn be32(bytes: &[u8], off: usize) -> Option<u32> {
    let b = bytes.get(off..off + 4)?;
    Some(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}

fn align4(n: usize) -> usize {
    (n + 3) & !3
}

fn read_u64_cells(val: &[u8]) -> Option<u64> {
    match val.len() {
        4 => Some(u32::from_be_bytes([val[0], val[1], val[2], val[3]]) as u64),
        8 => Some(u64::from_be_bytes([
            val[0], val[1], val[2], val[3], val[4], val[5], val[6], val[7],
        ])),
        _ => None,
    }
}

/// Return `(initrd_start, initrd_end)` physical addresses from `/chosen`.
pub fn initrd_range(fdt_phys: usize) -> Option<(usize, usize)> {
    if fdt_phys == 0 {
        return None;
    }
    // Header is 40 bytes; totalsize tells us how much is safe to map as a slice.
    let hdr = unsafe { core::slice::from_raw_parts(fdt_phys as *const u8, 40) };
    if be32(hdr, 0)? != FDT_MAGIC {
        return None;
    }
    let totalsize = be32(hdr, 4)? as usize;
    if totalsize < 40 || totalsize > 2 * 1024 * 1024 {
        return None;
    }
    let blob = unsafe { core::slice::from_raw_parts(fdt_phys as *const u8, totalsize) };
    let off_struct = be32(blob, 8)? as usize;
    let off_strings = be32(blob, 12)? as usize;
    let size_struct = be32(blob, 36)? as usize;
    if off_struct + size_struct > totalsize || off_strings >= totalsize {
        return None;
    }

    let mut off = off_struct;
    let end = off_struct + size_struct;
    let mut depth: i32 = 0;
    let mut in_chosen = false;
    let mut start: Option<u64> = None;
    let mut iend: Option<u64> = None;

    while off + 4 <= end {
        let token = be32(blob, off)?;
        off += 4;
        match token {
            FDT_BEGIN_NODE => {
                let name_start = off;
                while off < end && blob[off] != 0 {
                    off += 1;
                }
                if off >= end {
                    return None;
                }
                let name = core::str::from_utf8(&blob[name_start..off]).unwrap_or("");
                off = align4(off + 1);
                depth += 1;
                // Root is "" at depth 1; `/chosen` is name "chosen" at depth 2.
                in_chosen = depth == 2 && name == "chosen";
            }
            FDT_END_NODE => {
                if in_chosen {
                    in_chosen = false;
                }
                depth -= 1;
                if depth < 0 {
                    return None;
                }
            }
            FDT_PROP => {
                let len = be32(blob, off)? as usize;
                let nameoff = be32(blob, off + 4)? as usize;
                off += 8;
                if off + len > end || off_strings + nameoff >= totalsize {
                    return None;
                }
                let name_bytes = &blob[off_strings + nameoff..];
                let name_end = name_bytes.iter().position(|&c| c == 0).unwrap_or(0);
                let pname = core::str::from_utf8(&name_bytes[..name_end]).unwrap_or("");
                let val = &blob[off..off + len];
                off = align4(off + len);
                if in_chosen {
                    if pname == "linux,initrd-start" {
                        start = read_u64_cells(val);
                    } else if pname == "linux,initrd-end" {
                        iend = read_u64_cells(val);
                    }
                }
            }
            FDT_NOP => {}
            FDT_END => break,
            _ => return None,
        }
    }

    match (start, iend) {
        (Some(s), Some(e)) if e > s => Some((s as usize, e as usize)),
        _ => None,
    }
}
