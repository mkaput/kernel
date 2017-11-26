//! Memory subsystem

use multiboot2;

/// Initializes memory subsystem.
///
/// **KIO subsystem is required to be at least early initialized.**
///
/// **This function should be called only once.**
pub unsafe fn init(boot_info: &multiboot2::BootInformation) {
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");

    let available_bytes: u64 = memory_map_tag.memory_areas().map(|area| area.length).sum();
    kprintln!("available memory: {} bytes", available_bytes);
}
