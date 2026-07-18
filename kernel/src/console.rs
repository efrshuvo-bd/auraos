//! Early kernel console over UART.

use crate::uart;

pub fn print(s: &str) {
    uart::write_bytes(s.as_bytes());
}

pub fn println(s: &str) {
    print(s);
    print("\n");
}
