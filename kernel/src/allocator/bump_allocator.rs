use core::alloc::GlobalAlloc;

use crate::allocator::Locked;

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_size + heap_start;
        self.next = heap_start;
        self.allocations = 0;
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut bump = self.inner.lock();
        let alloc_start = (bump.next + layout.align() - 1) & !(layout.align() - 1);
        let alloc_end = alloc_start + layout.size();

        if alloc_end > bump.heap_end {
            return core::ptr::null_mut();
        }

        let ptr = alloc_start as *mut u8;
        bump.next = alloc_end;

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let mut bump = self.inner.lock();
        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}
