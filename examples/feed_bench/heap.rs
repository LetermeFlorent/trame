use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

static LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);

pub struct CountingHeap;

pub fn live_bytes() -> usize {
    LIVE_BYTES.load(Ordering::Relaxed)
}

fn counted(block: *mut u8, size: usize) -> *mut u8 {
    if !block.is_null() {
        LIVE_BYTES.fetch_add(size, Ordering::Relaxed);
    }
    block
}

unsafe impl GlobalAlloc for CountingHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        counted(unsafe { System.alloc(layout) }, layout.size())
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        counted(unsafe { System.alloc_zeroed(layout) }, layout.size())
    }

    unsafe fn dealloc(&self, block: *mut u8, layout: Layout) {
        unsafe { System.dealloc(block, layout) };
        LIVE_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
    }

    unsafe fn realloc(&self, block: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let moved = counted(unsafe { System.realloc(block, layout, new_size) }, new_size);
        if !moved.is_null() {
            LIVE_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
        }
        moved
    }
}
