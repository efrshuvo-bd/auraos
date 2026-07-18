//! PL011 UART for QEMU virt (base 0x0900_0000).

const UART0: usize = 0x0900_0000;
const UART_DR: usize = UART0;
const UART_FR: usize = UART0 + 0x18;
const UART_FR_TXFF: u32 = 1 << 5;

pub fn init() {
    // QEMU PL011 is usable without heavy init for early console.
}

fn putc(c: u8) {
    unsafe {
        while (core::ptr::read_volatile(UART_FR as *const u32) & UART_FR_TXFF) != 0 {}
        core::ptr::write_volatile(UART_DR as *mut u32, c as u32);
    }
}

pub fn write_bytes(bytes: &[u8]) {
    for &b in bytes {
        if b == b'\n' {
            putc(b'\r');
        }
        putc(b);
    }
}
