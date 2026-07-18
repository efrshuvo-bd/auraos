//! In-kernel IPC mailbox for early Agent Core ↔ shell messaging.

use core::sync::atomic::{AtomicU64, Ordering};

const MAILBOXES: usize = 16;

static mut MAILBOX: [u64; MAILBOXES] = [0; MAILBOXES];
static MSG_COUNT: AtomicU64 = AtomicU64::new(0);

pub fn init() {
    MSG_COUNT.store(0, Ordering::SeqCst);
    unsafe {
        for m in MAILBOX.iter_mut() {
            *m = 0;
        }
    }
}

pub fn send(channel: u32, payload: u64) -> bool {
    let ch = channel as usize;
    if ch >= MAILBOXES {
        return false;
    }
    unsafe {
        MAILBOX[ch] = payload;
    }
    MSG_COUNT.fetch_add(1, Ordering::Relaxed);
    true
}

pub fn recv(channel: u32) -> u64 {
    let ch = channel as usize;
    if ch >= MAILBOXES {
        return 0;
    }
    unsafe { MAILBOX[ch] }
}

pub fn messages() -> u64 {
    MSG_COUNT.load(Ordering::Relaxed)
}
