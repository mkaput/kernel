//! Memory management subsystem

pub mod paging;

use core::cmp;

use multiboot2;

use self::paging::{Frame, FrameAlloc, CoreFrameAlloc};

/// Initializes memory subsystem.
///
/// **KIO subsystem is required to be at least early initialized.**
///
/// **This function should be called only once.**
pub unsafe fn init(boot_info: &multiboot2::BootInformation) {
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");

    let available_bytes: u64 = memory_map_tag.memory_areas().map(|area| area.length).sum();
    kprintln!("available memory: {} bytes", available_bytes);

    let elf_sections_tag = boot_info
        .elf_sections_tag()
        .expect("Elf-sections tag required");

    let (kernel_start, kernel_end) = elf_sections_tag
        .sections()
        .map(|s| (s.start_address(), s.end_address()))
        .fold(
            (usize::max_value(), usize::min_value()),
            |(accs, acce), (s, e)| (cmp::min(accs, s), cmp::max(acce, e)),
        );

    let multiboot_start = boot_info.start_address();
    let multiboot_end = boot_info.end_address();

    let reserved = [
        (Frame::containing_address(kernel_start), Frame::containing_address(kernel_end)),
        (Frame::containing_address(multiboot_start), Frame::containing_address(multiboot_end)),
    ];

    let mut frame_allocator = CoreFrameAlloc::new(
        memory_map_tag.memory_areas(),
        &reserved
    );

    for i in 0.. {
        if let None = frame_allocator.alloc() {
            kprintln!("allocated {} frames", i);
            break;
        }
    }
}
