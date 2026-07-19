//! Newc (SVR4) CPIO archive walker for initrd guest ELF lookup.

const HEADER_LEN: usize = 110;
const MAGIC: &[u8] = b"070701";

fn hex_u32(s: &[u8]) -> Option<u32> {
    if s.len() != 8 {
        return None;
    }
    let mut v = 0u32;
    for &c in s {
        v <<= 4;
        v |= match c {
            b'0'..=b'9' => (c - b'0') as u32,
            b'a'..=b'f' => (c - b'a' + 10) as u32,
            b'A'..=b'F' => (c - b'A' + 10) as u32,
            _ => return None,
        };
    }
    Some(v)
}

fn align4(n: usize) -> usize {
    (n + 3) & !3
}

/// Find a regular file by exact path/name inside a newc CPIO image.
pub fn lookup<'a>(archive: &'a [u8], name: &str) -> Option<&'a [u8]> {
    let mut off = 0usize;
    while off + HEADER_LEN <= archive.len() {
        let hdr = &archive[off..off + HEADER_LEN];
        if &hdr[0..6] != MAGIC {
            return None;
        }
        let namesize = hex_u32(&hdr[94..102])? as usize;
        let filesize = hex_u32(&hdr[54..62])? as usize;
        let name_off = off + HEADER_LEN;
        let name_end = name_off + namesize;
        if name_end > archive.len() {
            return None;
        }
        // namesize includes trailing NUL
        let entry_name = core::str::from_utf8(&archive[name_off..name_end - 1]).ok()?;
        let data_off = align4(name_end);
        let data_end = data_off.checked_add(filesize)?;
        if data_end > archive.len() {
            return None;
        }
        if entry_name == "TRAILER!!!" {
            return None;
        }
        if entry_name == name || entry_name.strip_prefix("./") == Some(name) {
            return Some(&archive[data_off..data_end]);
        }
        off = align4(data_end);
    }
    None
}
