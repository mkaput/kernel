//! Memory management subsystem

pub mod alloc;
pub mod paging;
pub mod stack;

use core::cmp;

use multiboot2::BootInformation;

use super::HEAP_ALLOCATOR;

use self::paging::{remap_kernel, ActivePageTable, CoreFrameAlloc, Frame, Page};
use self::stack::{Stack, StackAllocator};

pub(super) const HEAP_START: usize = 0o_000_004_000_000_0000;
pub(super) const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

const STACK_PAGES: usize = 100;

static mut FRAME_ALLOC: Option<CoreFrameAlloc> = None;
static mut ACTIVE_PAGE_TABLE: Option<ActivePageTable> = None;
static mut STACK_ALLOCATOR: Option<StackAllocator> = None;

/// Initializes memory subsystem.
///
/// **KIO subsystem is required to be at least early initialized.**
///
/// **This function should be called only once.**
pub unsafe fn init(boot_info: &BootInformation) {
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");

    let reserved_frames = {
        let elf_sections_tag = boot_info
            .elf_sections_tag()
            .expect("Elf sections tag required");

        let (kernel_start, kernel_end) = elf_sections_tag
            .sections()
            .filter(|s| s.is_allocated())
            .map(|s| (s.start_address(), s.end_address()))
            .fold(
                (usize::max_value(), usize::min_value()),
                |(accs, acce), (s, e)| (cmp::min(accs, s), cmp::max(acce, e)),
            );

        let multiboot_start = boot_info.start_address();
        let multiboot_end = boot_info.end_address();

        [
            (
                Frame::containing_address(kernel_start),
                Frame::containing_address(kernel_end),
            ),
            (
                Frame::containing_address(multiboot_start),
                Frame::containing_address(multiboot_end),
            ),
        ]
    };

    let mut frame_alloc = CoreFrameAlloc::new(memory_map_tag.memory_areas(), reserved_frames);

    let mut active_table = remap_kernel(&mut frame_alloc, boot_info);

    let heap_start_page = Page::containing_address(HEAP_START);
    let heap_end_page = Page::containing_address(HEAP_START + HEAP_SIZE - 1);

    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        active_table.map(page, paging::EntryFlags::WRITABLE, &mut frame_alloc);
    }

    HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);

    let stack_start_page = heap_end_page + 1;
    let stack_end_page = stack_start_page + STACK_PAGES;
    let stack_alloc = StackAllocator::new(Page::range_inclusive(stack_start_page, stack_end_page));

    FRAME_ALLOC = Some(frame_alloc);
    ACTIVE_PAGE_TABLE = Some(active_table);
    STACK_ALLOCATOR = Some(stack_alloc);
}

/// Allocates new stack from global stack pool.
///
/// **This function is not thread safe!**
pub unsafe fn alloc_stack(size_in_pages: usize) -> Option<Stack> {
    let stack_allocator = STACK_ALLOCATOR.as_mut().unwrap();
    let active_table = ACTIVE_PAGE_TABLE.as_mut().unwrap();
    let frame_alloc = FRAME_ALLOC.as_mut().unwrap();
    stack_allocator.alloc(active_table, frame_alloc, size_in_pages)
}
