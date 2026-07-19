//! Display foundations: VirtIO-GPU probe + QEMU ramfb smoke test.
//!
//! Sprint 5 (SCRUM-29): scan VirtIO-MMIO for GPU (device id 16), and when QEMU
//! exposes `etc/ramfb` via fw_cfg, map a 480×800 XRGB8888 surface, solid-fill,
//! and draw a few text glyphs (matches host `userspace/shell` visual contract).

use crate::console;
use crate::frame;
use crate::virtio;

/// Mobile-shaped surface (same as host PPM sketch).
const FB_WIDTH: u32 = 480;
const FB_HEIGHT: u32 = 800;
const FB_BPP: u32 = 4;
const FB_STRIDE: u32 = FB_WIDTH * FB_BPP;
const FB_BYTES: usize = (FB_WIDTH * FB_HEIGHT * FB_BPP) as usize;

/// DRM_FORMAT_XRGB8888 ('XR24'), big-endian fourcc as stored in ramfb cfg.
const FOURCC_XR24: u32 = 0x3432_5258;

/// QEMU virt fw_cfg MMIO (within identity-mapped device window).
const FW_CFG_BASE: usize = 0x0902_0000;
const FW_CFG_DATA: usize = FW_CFG_BASE;
const FW_CFG_SELECTOR: usize = FW_CFG_BASE + 0x08;
const FW_CFG_FILE_DIR: u16 = 0x19;

const VIRTIO_ID_GPU: u32 = 16;

pub fn init() {
    probe_virtio_gpu();
    match setup_ramfb() {
        Ok(addr) => {
            smoke_draw(addr);
            console::println("display: ramfb smoke ok (solid fill + glyphs)");
        }
        Err(msg) => {
            console::print("display: ramfb skip - ");
            console::println(msg);
        }
    }
}

fn probe_virtio_gpu() {
    match virtio::find_device(VIRTIO_ID_GPU) {
        Some((base, version)) => {
            console::print("display: virtio-gpu at mmio ");
            print_hex_usize(base);
            console::print(" version=");
            console::print(match version {
                1 => "1 (legacy)",
                2 => "2 (modern)",
                _ => "?",
            });
            console::println(" (probe only; queues deferred)");
        }
        None => console::println("display: no virtio-gpu device"),
    }
}

fn setup_ramfb() -> Result<usize, &'static str> {
    let select = find_fw_cfg_file(b"etc/ramfb").ok_or("no etc/ramfb (use run-qemu-gui.ps1)")?;
    let pages = (FB_BYTES + frame::PAGE_SIZE - 1) / frame::PAGE_SIZE;
    let addr = alloc_contig_pages(pages).ok_or("framebuffer alloc failed")?;

    // Zero was done per-page; fill comes next in smoke_draw.
    let mut cfg = [0u8; 28];
    write_be64(&mut cfg[0..8], addr as u64);
    write_be32(&mut cfg[8..12], FOURCC_XR24);
    write_be32(&mut cfg[12..16], 0); // flags
    write_be32(&mut cfg[16..20], FB_WIDTH);
    write_be32(&mut cfg[20..24], FB_HEIGHT);
    write_be32(&mut cfg[24..28], FB_STRIDE);

    fw_cfg_select(select);
    fw_cfg_write(&cfg);

    console::print("display: ramfb mapped ");
    print_u32(FB_WIDTH);
    console::print("x");
    print_u32(FB_HEIGHT);
    console::print(" @ ");
    print_hex_usize(addr);
    console::println("");
    Ok(addr)
}

fn smoke_draw(base: usize) {
    // XRGB8888 colors (host framebuffer atmosphere cue).
    let fill = 0x0018_5a6eu32;
    unsafe {
        let px = base as *mut u32;
        let n = (FB_WIDTH * FB_HEIGHT) as usize;
        for i in 0..n {
            core::ptr::write_volatile(px.add(i), fill);
        }
        // Brand header bar
        fill_rect(base, 0, 0, FB_WIDTH, 72, 0x000c_5a6e);
        draw_text(base, 24, 28, b"AURAOS", 0x00f0_faff);
        draw_text(base, 24, 120, b"HOME", 0x00e6_f0f5);
        draw_text(base, 24, 150, b"AGENT ALWAYS ON", 0x00b4_d2dc);
        // Agent panel (bottom third)
        fill_rect(base, 16, FB_HEIGHT - 280, FB_WIDTH - 32, 248, 0x0012_1c24);
        draw_text(base, 32, FB_HEIGHT - 250, b"AGENT CORE", 0x0078_dcc8);
        draw_text(base, 32, FB_HEIGHT - 210, b"ASK ANYTHING...", 0x00a0_aab4);
        draw_text(base, 32, FB_HEIGHT - 160, b"[HELP] [STATUS]", 0x0064_b4a0);
    }
}

unsafe fn fill_rect(base: usize, x0: u32, y0: u32, w: u32, h: u32, color: u32) {
    let px = base as *mut u32;
    let x1 = (x0 + w).min(FB_WIDTH);
    let y1 = (y0 + h).min(FB_HEIGHT);
    let mut y = y0;
    while y < y1 {
        let mut x = x0;
        while x < x1 {
            core::ptr::write_volatile(px.add((y * FB_WIDTH + x) as usize), color);
            x += 1;
        }
        y += 1;
    }
}

unsafe fn draw_text(base: usize, x: u32, y: u32, text: &[u8], color: u32) {
    let mut cx = x;
    for &ch in text {
        draw_char(base, cx, y, ch, color);
        cx += 10;
        if cx + 8 >= FB_WIDTH {
            break;
        }
    }
}

unsafe fn draw_char(base: usize, x: u32, y: u32, ch: u8, color: u32) {
    let glyph = glyph_for(ch);
    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..5u32 {
            if bits & (1 << (4 - col)) != 0 {
                for dy in 0..2u32 {
                    for dx in 0..2u32 {
                        let px = x + col * 2 + dx;
                        let py = y + (row as u32) * 2 + dy;
                        if px < FB_WIDTH && py < FB_HEIGHT {
                            let p = (base as *mut u32).add((py * FB_WIDTH + px) as usize);
                            core::ptr::write_volatile(p, color);
                        }
                    }
                }
            }
        }
    }
}

fn glyph_for(ch: u8) -> [u8; 5] {
    match ch.to_ascii_uppercase() {
        b'A' => [0b01110, 0b10001, 0b11111, 0b10001, 0b10001],
        b'C' => [0b01111, 0b10000, 0b10000, 0b10000, 0b01111],
        b'E' => [0b11111, 0b10000, 0b11110, 0b10000, 0b11111],
        b'G' => [0b01111, 0b10000, 0b10111, 0b10001, 0b01111],
        b'H' => [0b10001, 0b10001, 0b11111, 0b10001, 0b10001],
        b'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        b'M' => [0b10001, 0b11011, 0b10101, 0b10001, 0b10001],
        b'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001],
        b'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b01110],
        b'P' => [0b11110, 0b10001, 0b11110, 0b10000, 0b10000],
        b'R' => [0b11110, 0b10001, 0b11110, 0b10010, 0b10001],
        b'S' => [0b01111, 0b10000, 0b01110, 0b00001, 0b11110],
        b'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100],
        b'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        b'W' => [0b10001, 0b10001, 0b10101, 0b11011, 0b10001],
        b'Y' => [0b10001, 0b01010, 0b00100, 0b00100, 0b00100],
        b' ' => [0, 0, 0, 0, 0],
        b'.' => [0, 0, 0, 0b00100, 0b00100],
        b'[' => [0b01110, 0b01000, 0b01000, 0b01000, 0b01110],
        b']' => [0b01110, 0b00010, 0b00010, 0b00010, 0b01110],
        _ => [0b11111, 0b10001, 0b10001, 0b10001, 0b11111],
    }
}

fn alloc_contig_pages(pages: usize) -> Option<usize> {
    let first = frame::alloc_frame()?;
    for i in 1..pages {
        let p = frame::alloc_frame()?;
        if p != first + i * frame::PAGE_SIZE {
            return None;
        }
    }
    Some(first)
}

fn find_fw_cfg_file(name: &[u8]) -> Option<u16> {
    fw_cfg_select(FW_CFG_FILE_DIR);
    let count = fw_cfg_read_be32();
    if count == 0 || count > 64 {
        return None;
    }
    for _ in 0..count {
        let _size = fw_cfg_read_be32();
        let select = fw_cfg_read_be16();
        let _reserved = fw_cfg_read_be16();
        let mut fname = [0u8; 56];
        for b in &mut fname {
            *b = fw_cfg_read8();
        }
        let end = fname.iter().position(|&c| c == 0).unwrap_or(56);
        if &fname[..end] == name {
            return Some(select);
        }
    }
    None
}

fn fw_cfg_select(key: u16) {
    unsafe {
        // Selector is big-endian on MMIO.
        let be = key.to_be();
        core::ptr::write_volatile(FW_CFG_SELECTOR as *mut u16, be);
    }
}

fn fw_cfg_read8() -> u8 {
    unsafe { core::ptr::read_volatile(FW_CFG_DATA as *const u8) }
}

fn fw_cfg_read_be16() -> u16 {
    let b0 = fw_cfg_read8() as u16;
    let b1 = fw_cfg_read8() as u16;
    (b0 << 8) | b1
}

fn fw_cfg_read_be32() -> u32 {
    let b0 = fw_cfg_read8() as u32;
    let b1 = fw_cfg_read8() as u32;
    let b2 = fw_cfg_read8() as u32;
    let b3 = fw_cfg_read8() as u32;
    (b0 << 24) | (b1 << 16) | (b2 << 8) | b3
}

fn fw_cfg_write(bytes: &[u8]) {
    unsafe {
        for &b in bytes {
            core::ptr::write_volatile(FW_CFG_DATA as *mut u8, b);
        }
    }
}

fn write_be32(dst: &mut [u8], v: u32) {
    dst[0] = (v >> 24) as u8;
    dst[1] = (v >> 16) as u8;
    dst[2] = (v >> 8) as u8;
    dst[3] = v as u8;
}

fn write_be64(dst: &mut [u8], v: u64) {
    for i in 0..8 {
        dst[i] = (v >> (56 - i * 8)) as u8;
    }
}

fn print_hex_usize(v: usize) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    console::print("0x");
    let mut started = false;
    for i in (0..16).rev() {
        let nibble = ((v >> (i * 4)) & 0xf) as usize;
        if nibble != 0 || started || i == 0 {
            started = true;
            let b = [HEX[nibble]];
            crate::uart::write_bytes(&b);
        }
    }
}

fn print_u32(mut v: u32) {
    if v == 0 {
        console::print("0");
        return;
    }
    let mut buf = [0u8; 10];
    let mut i = 10;
    while v > 0 {
        i -= 1;
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
    }
    crate::uart::write_bytes(&buf[i..]);
}
