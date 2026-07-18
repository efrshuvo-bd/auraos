//! Simple bump allocator heap for kernel `alloc`.

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

const HEAP_SIZE: usize = 512 * 1024;

#[repr(C, align(16))]
struct HeapMemory([u8; HEAP_SIZE]);

struct Heap(UnsafeCell<HeapMemory>);

unsafe impl Sync for Heap {}

static HEAP: Heap = Heap(UnsafeCell::new(HeapMemory([0; HEAP_SIZE])));
static HEAP_POS: AtomicUsize = AtomicUsize::new(0);

struct BumpAllocator;

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        loop {
            let pos = HEAP_POS.load(Ordering::Relaxed);
            let aligned = (pos + align - 1) & !(align - 1);
            let Some(next) = aligned.checked_add(size) else {
                return core::ptr::null_mut();
            };
            if next > HEAP_SIZE {
                return core::ptr::null_mut();
            }
            if HEAP_POS
                .compare_exchange(pos, next, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                let base = (&*HEAP.0.get()).0.as_ptr() as usize;
                return (base + aligned) as *mut u8;
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator;

pub fn init_heap() {
    HEAP_POS.store(0, Ordering::SeqCst);
}

pub fn heap_used() -> usize {
    HEAP_POS.load(Ordering::Relaxed)
}
