use core::alloc::GlobalAlloc;

use crate::allocator::Locked;

pub struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    pub fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    pub fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

pub struct LinkedListAllocator {
    head: ListNode,
}
impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            head: ListNode {
                size: 0,
                next: None,
            },
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        unsafe {
            self.add_free_region(heap_start, heap_size);
        }
    }
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        assert!(addr % core::mem::align_of::<ListNode>() == 0);
        assert!(size >= core::mem::size_of::<ListNode>());

        let node = addr as *mut ListNode;

        unsafe {
            node.write(ListNode {
                size,
                next: self.head.next.take(),
            });

            self.head.next = Some(&mut *node);
        }
    }

    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        let mut current = &mut self.head;
        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = alloc_from_region(region, size, align) {
                let next = region.next.take();
                let found = current.next.take().unwrap();
                current.next = next;
                return Some((found, alloc_start));
            } else {
                current = current.next.as_mut().unwrap();
            }
        }
        None
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

fn size_align(layout: core::alloc::Layout) -> (usize, usize) {
    let layout = layout
        .align_to(core::mem::align_of::<ListNode>())
        .expect("adjusting alignment failed")
        .pad_to_align();

    let size = layout.size().max(core::mem::size_of::<ListNode>());
    (size, layout.align())
}

fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
    let alloc_start = align_up(region.start_addr(), align);
    let alloc_end = alloc_start.checked_add(size).ok_or(())?;

    if alloc_end > region.end_addr() {
        return Err(());
    }

    let excess_size = region.end_addr() - alloc_end;

    if excess_size > 0 && excess_size < core::mem::size_of::<ListNode>() {
        return Err(());
    }

    Ok(alloc_start)
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut allocator = self.lock();

        let (size, align) = size_align(layout);

        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = match alloc_start.checked_add(size) {
                Some(end) => end,
                None => return core::ptr::null_mut(),
            };

            let region_end = region.end_addr();
            let excess_size = region_end - alloc_end;

            if excess_size > 0 {
                unsafe {
                    allocator.add_free_region(alloc_end, excess_size);
                }
            }

            alloc_start as *mut u8
        } else {
            core::ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let mut allocator = self.lock();
        let (size, _) = size_align(layout);
        unsafe {
            allocator.add_free_region(ptr as usize, size);
        }
    }
}
