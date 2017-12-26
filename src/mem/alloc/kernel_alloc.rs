use alloc::heap::{Alloc, AllocErr, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

use super::util::align_up;

// TODO: Implement this to replace linked_list_allocator

#[derive(Debug)]
pub struct KernelAlloc {
    heap_start: usize,
    heap_end: usize,
    next: AtomicUsize,
}

impl KernelAlloc {
    pub const fn new(heap_start: usize, heap_end: usize) -> KernelAlloc {
        KernelAlloc {
            heap_start,
            heap_end,
            next: AtomicUsize::new(heap_start),
        }
    }
}

unsafe impl<'a> Alloc for &'a KernelAlloc {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        loop {
            let old_next = self.next.load(Ordering::Relaxed);
            let alloc_start = align_up(old_next, layout.align());
            let alloc_end = alloc_start.saturating_add(layout.size());

            if alloc_end <= self.heap_end {
                let new_next = self.next
                    .compare_and_swap(old_next, alloc_end, Ordering::Relaxed);
                if new_next == old_next {
                    return Ok(alloc_start as *mut u8);
                }
            } else {
                return Err(AllocErr::Exhausted { request: layout });
            }
        }
    }

    unsafe fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
        println!("DEALLOCATING MEMORY IS NOT IMPLEMENTED YET");
    }
}
