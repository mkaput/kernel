//! Memory management subsystem

pub mod alloc;
pub mod paging;

use core::cmp;

use multiboot2::BootInformation;

use super::HEAP_ALLOCATOR;

use self::paging::{remap_kernel, CoreFrameAlloc, Frame, Page};

pub const HEAP_START: usize = 0o_000_004_000_000_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

/// Initializes memory subsystem.
///
/// **KIO subsystem is required to be at least early initialized.**
///
/// **This function should be called only once.**
pub unsafe fn init(boot_info: &BootInformation) {
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");

    let reserved = {
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

    let mut frame_allocator = CoreFrameAlloc::new(memory_map_tag.memory_areas(), &reserved);

    let mut active_table = remap_kernel(&mut frame_allocator, boot_info);

    let heap_start_page = Page::containing_address(HEAP_START);
    let heap_end_page = Page::containing_address(HEAP_START + HEAP_SIZE - 1);

    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        active_table.map(page, paging::EntryFlags::WRITABLE, &mut frame_allocator);
    }

    HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
}
