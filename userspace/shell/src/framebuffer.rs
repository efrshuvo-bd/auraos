//! Software framebuffer demo (PPM) — VirtIO-GPU stand-in for Phase 4.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

const W: usize = 480;
const H: usize = 800;

pub fn render_home_with_agent(path: &str) -> Result<()> {
    let mut px = vec![0u8; W * H * 3];
    // Atmosphere gradient (deep teal → night)
    for y in 0..H {
        for x in 0..W {
            let t = y as f32 / H as f32;
            let r = (8.0 + 20.0 * t) as u8;
            let g = (40.0 + 30.0 * (1.0 - t)) as u8;
            let b = (55.0 + 40.0 * t) as u8;
            set(&mut px, x, y, r, g, b);
        }
    }
    // Brand header bar
    fill_rect(&mut px, 0, 0, W, 72, 12, 90, 110);
    draw_text_approx(&mut px, 24, 28, "AuraOS", 240, 250, 255);
    // Home headline
    draw_text_approx(&mut px, 24, 120, "Home", 230, 240, 245);
    draw_text_approx(&mut px, 24, 150, "Agent always on", 180, 210, 220);
    // Agent overlay panel (bottom third)
    fill_rect(&mut px, 16, H - 280, W - 32, 248, 18, 28, 36);
    draw_text_approx(&mut px, 32, H - 250, "Agent Core", 120, 220, 200);
    draw_text_approx(&mut px, 32, H - 210, "Ask anything...", 160, 170, 180);
    draw_text_approx(&mut px, 32, H - 160, "[help] [status] [services]", 100, 180, 160);

    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    write_ppm(path, &px)?;
    Ok(())
}

fn set(buf: &mut [u8], x: usize, y: usize, r: u8, g: u8, b: u8) {
    if x >= W || y >= H {
        return;
    }
    let i = (y * W + x) * 3;
    buf[i] = r;
    buf[i + 1] = g;
    buf[i + 2] = b;
}

fn fill_rect(buf: &mut [u8], x0: usize, y0: usize, w: usize, h: usize, r: u8, g: u8, b: u8) {
    for y in y0..(y0 + h).min(H) {
        for x in x0..(x0 + w).min(W) {
            set(buf, x, y, r, g, b);
        }
    }
}

/// Tiny 3x5 block-font style glyphs for demo labels (A-Z, 0-9, space, punctuation).
fn draw_text_approx(buf: &mut [u8], x: usize, y: usize, text: &str, r: u8, g: u8, b: u8) {
    let mut cx = x;
    for ch in text.chars() {
        draw_char(buf, cx, y, ch, r, g, b);
        cx += 10;
        if cx + 8 >= W {
            break;
        }
    }
}

fn draw_char(buf: &mut [u8], x: usize, y: usize, ch: char, r: u8, g: u8, b: u8) {
    let glyph = glyph_for(ch);
    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..5 {
            if bits & (1 << (4 - col)) != 0 {
                for dy in 0..2 {
                    for dx in 0..2 {
                        set(buf, x + col * 2 + dx, y + row * 2 + dy, r, g, b);
                    }
                }
            }
        }
    }
}

fn glyph_for(ch: char) -> [u8; 5] {
    match ch.to_ascii_uppercase() {
        'A' => [0b01110, 0b10001, 0b11111, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b11110, 0b10001, 0b11110],
        'C' => [0b01111, 0b10000, 0b10000, 0b10000, 0b01111],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' => [0b11111, 0b10000, 0b11110, 0b10000, 0b11111],
        'G' => [0b01111, 0b10000, 0b10111, 0b10001, 0b01111],
        'H' => [0b10001, 0b10001, 0b11111, 0b10001, 0b10001],
        'I' => [0b11111, 0b00100, 0b00100, 0b00100, 0b11111],
        'K' => [0b10001, 0b10010, 0b11100, 0b10010, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10001, 0b10001],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b11110, 0b10000, 0b10000],
        'R' => [0b11110, 0b10001, 0b11110, 0b10010, 0b10001],
        'S' => [0b01111, 0b10000, 0b01110, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10101, 0b11011, 0b10001],
        'Y' => [0b10001, 0b01010, 0b00100, 0b00100, 0b00100],
        ' ' => [0, 0, 0, 0, 0],
        '.' => [0, 0, 0, 0b00100, 0b00100],
        '[' => [0b01110, 0b01000, 0b01000, 0b01000, 0b01110],
        ']' => [0b01110, 0b00010, 0b00010, 0b00010, 0b01110],
        _ => [0b11111, 0b10001, 0b10001, 0b10001, 0b11111],
    }
}

fn write_ppm(path: &str, px: &[u8]) -> Result<()> {
    let mut out = format!("P6\n{W} {H}\n255\n").into_bytes();
    out.extend_from_slice(px);
    fs::write(path, out).with_context(|| format!("write {path}"))?;
    Ok(())
}
