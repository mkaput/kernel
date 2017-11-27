mod active_page_table;
mod frame;
mod frame_alloc;
mod inactive_page_table;
mod mapper;
mod page;
mod page_table;
mod tmp_page;

use multiboot2::BootInformation;

use drv::gfx::vga::text_buffer::VGA_TEXT_BUFFER_ADDR;

use self::active_page_table::ActivePageTable;
use self::inactive_page_table::InactivePageTable;
use self::page::Page;
use self::page_table::*;
use self::page_table::EntryFlags as F;
use self::tmp_page::TmpPage;

pub use self::frame::*;
pub use self::frame_alloc::*;

pub type VirtualAddress = usize;
pub type PhysicalAddress = usize;

const PAGE_SIZE: usize = 4096;

const REMAP_TMP_PAGE_NUMBER: usize = 0xdeadbeef;

pub fn remap_kernel(allocator: &mut impl FrameAlloc, boot_info: &BootInformation) {
    let mut tmp_page = TmpPage::new(
        Page {
            number: REMAP_TMP_PAGE_NUMBER,
        },
        allocator,
    );

    let mut active_table = unsafe { ActivePageTable::new() };

    let mut new_table = {
        let frame = allocator.alloc().expect("out of memory");
        InactivePageTable::new(frame, &mut active_table, &mut tmp_page)
    };

    active_table.with(&mut new_table, &mut tmp_page, |mapper| {
        kprintln!("mapping sections:");

        let elf_sections_tag = boot_info
            .elf_sections_tag()
            .expect("Elf sections tag required");

        let string_table = elf_sections_tag.string_table();

        for section in elf_sections_tag.sections() {
            if !section.is_allocated() {
                // Skip not allocated sections
                continue;
            }

            assert_eq!(
                section.start_address() % PAGE_SIZE,
                0,
                "kernel sections need to be page aligned"
            );

            let flags = EntryFlags::from_elf_section_flags(section);

            let start_frame = Frame::containing_address(section.start_address());
            let end_frame = Frame::containing_address(section.end_address() - 1);

            let section_name = string_table.section_name(section);
            kprintln!("  {:-16} {:#x}-{:#x}", section_name, start_frame.start_address(), end_frame.end_address());

            for frame in Frame::range_inclusive(start_frame, end_frame) {
                mapper.identity_map(frame, flags, allocator);
            }
        }

        // Identity map VGA text buffer
        let vga_buffer_frame = Frame::containing_address(VGA_TEXT_BUFFER_ADDR);
        kprintln!("  VGA text buffer  {:#x}-{:#x}", vga_buffer_frame.start_address(), vga_buffer_frame.end_address());
        mapper.identity_map(vga_buffer_frame, F::WRITABLE, allocator);

        // Identity map Multiboot info
        let multiboot_start = Frame::containing_address(boot_info.start_address());
        let multiboot_end = Frame::containing_address(boot_info.end_address() - 1);
        kprintln!("  Boot info        {:#x}-{:#x}", multiboot_start.start_address(), multiboot_end.end_address());
        for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
            mapper.identity_map(frame, F::PRESENT, allocator);
        }
    });

    let old_table = active_table.switch(new_table);
    let old_p4_page = Page::containing_address(old_table.p4_frame.start_address());
    active_table.unmap(old_p4_page, allocator);

    kprintln!("remapped kernel successfully");
}
