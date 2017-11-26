mod core_frame_alloc;
mod page_table;

use core::ops::{Deref, DerefMut};
use core::ptr::Unique;

use multiboot2::BootInformation;

use x86_64::instructions::tlb;
use x86_64::registers::control_regs::{cr3, cr3_write};
use x86_64::PhysicalAddress as NPhysicalAddress;
use x86_64::VirtualAddress as NVirtualAddress;

use drv::gfx::vga::text_buffer::VGA_TEXT_BUFFER_ADDR;

use self::page_table::*;
use self::page_table::EntryFlags as F;

pub use self::core_frame_alloc::*;

pub type VirtualAddress = usize;
pub type PhysicalAddress = usize;

const PAGE_SIZE: usize = 4096;

const REMAP_TMP_PAGE_NUMBER: usize = 0xdeadbeef;

/// Represents physical memory frame.
///
/// Frames are comparable via their start offsets in physical memory.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Frame {
    number: usize,
}

impl Frame {
    // We deliberately do not implement Clone for Frame, because cloning frame
    // does not make real sense. This function is only implementation internal.
    const fn clone(&self) -> Frame {
        Frame {
            number: self.number,
        }
    }

    /// Returns `Frame` containing given physical `address`.
    pub const fn containing_address(address: PhysicalAddress) -> Frame {
        Frame {
            number: address / PAGE_SIZE,
        }
    }

    /// Returns frame start physical address.
    pub const fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }

    /// Returns frame end physical address (exclusive).
    pub const fn end_address(&self) -> PhysicalAddress {
        (self.number + 1) * PAGE_SIZE
    }

    fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        FrameIter { start, end }
    }
}


struct FrameIter {
    start: Frame,
    end: Frame,
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.number += 1;
            Some(frame)
        } else {
            None
        }
    }
}

/// Common interface for *frame allocators*.
///
/// Frame allocator is a routine which manages used and free frames,
/// it is responsible for providing free frames on request.
pub trait FrameAlloc {
    /// Allocate unused frame, or return `None` if all frames are used.
    fn alloc(&mut self) -> Option<Frame>;

    /// Return frame back to free frames pool.
    fn dealloc(&mut self, frame: Frame);
}

/// Represents virtual memory page
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Page {
    number: usize,
}

impl Page {
    /// Returns `Page` containing given virtual `address`
    ///
    /// ## Panics
    ///
    /// This function asserts that address' sign extension is valid.
    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(
            address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_0000,
            "invalid virtual address: 0x{:x}",
            address
        );
        Page {
            number: address / PAGE_SIZE,
        }
    }

    /// Returns page start virtual address.
    pub const fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }

    /// Returns page end virtual address (exclusive).
    pub const fn end_address(&self) -> PhysicalAddress {
        (self.number + 1) * PAGE_SIZE
    }

    fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }

    fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }

    fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }

    fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }
}

pub struct Mapper {
    p4: Unique<PageTable<L4>>,
}

impl Mapper {
    pub unsafe fn new() -> Mapper {
        Mapper {
            p4: Unique::new_unchecked(P4),
        }
    }

    fn p4(&self) -> &PageTable<L4> {
        unsafe { self.p4.as_ref() }
    }

    fn p4_mut(&mut self) -> &mut PageTable<L4> {
        unsafe { self.p4.as_mut() }
    }

    /// Translates specified virtual address to physical address using currently used page table.
    /// Returns `None` if the address is not mapped.
    pub fn translate(&self, virtual_address: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = virtual_address % PAGE_SIZE;
        self.translate_page(Page::containing_address(virtual_address))
            .map(|frame| frame.start_address() + offset)
    }

    fn translate_page(&self, page: Page) -> Option<Frame> {
        let p3 = self.p4().next_table(page.p4_index());
        let p2 = p3.and_then(|p3| p3.next_table(page.p3_index()));
        let p1 = p2.and_then(|p2| p2.next_table(page.p2_index()));
        let frame = p1.and_then(|p1| p1[page.p1_index()].pointed_frame());

        frame.or_else(|| {
            // Handle huge pages
            // TODO: How it works?
            p3.and_then(|p3| {
                let p3_entry = &p3[page.p3_index()];
                // 1GiB page?
                if let Some(start_frame) = p3_entry.pointed_frame() {
                    if p3_entry.flags().contains(F::HUGE_PAGE) {
                        // address must be 1GiB aligned
                        assert_eq!(start_frame.number % (ENTRY_COUNT * ENTRY_COUNT), 0);
                        return Some(Frame {
                            number: start_frame.number + page.p2_index() * ENTRY_COUNT
                                + page.p1_index(),
                        });
                    }
                }
                if let Some(p2) = p3.next_table(page.p3_index()) {
                    let p2_entry = &p2[page.p2_index()];
                    // 2MiB page?
                    if let Some(start_frame) = p2_entry.pointed_frame() {
                        if p2_entry.flags().contains(F::HUGE_PAGE) {
                            // address must be 2MiB aligned
                            assert_eq!(start_frame.number % ENTRY_COUNT, 0);
                            return Some(Frame {
                                number: start_frame.number + page.p1_index(),
                            });
                        }
                    }
                }
                None
            })
        })
    }

    /// Maps the page to the frame with the provided flags.
    /// The `PRESENT` flag is added by default. This function needs
    /// a `FrameAllocator` as it might need to create new page tables.
    pub fn map_to(
        &mut self,
        page: Page,
        frame: Frame,
        flags: EntryFlags,
        allocator: &mut impl FrameAlloc,
    ) {
        let p4 = self.p4_mut();
        let p3 = p4.next_table_create(page.p4_index(), allocator);
        let p2 = p3.next_table_create(page.p3_index(), allocator);
        let p1 = p2.next_table_create(page.p2_index(), allocator);

        assert!(p1[page.p1_index()].is_unused());
        p1[page.p1_index()].set(frame, flags | EntryFlags::PRESENT);
    }

    /// Maps the page to some free frame with the provided flags.
    /// The free frame is allocated from the given `FrameAllocator`.
    pub fn map(&mut self, page: Page, flags: EntryFlags, allocator: &mut impl FrameAlloc) {
        let frame = allocator.alloc().expect("out of memory");
        self.map_to(page, frame, flags, allocator)
    }

    /// Identity map the the given frame with the provided flags.
    /// The `FrameAllocator` is used to create new page tables if needed.
    pub fn identity_map(
        &mut self,
        frame: Frame,
        flags: EntryFlags,
        allocator: &mut impl FrameAlloc,
    ) {
        let page = Page::containing_address(frame.start_address());
        self.map_to(page, frame, flags, allocator)
    }

    /// Unmaps the given page and adds all freed frames to the given
    /// `FrameAllocator`.
    pub fn unmap(&mut self, page: Page, allocator: &mut impl FrameAlloc) {
        assert!(self.translate(page.start_address()).is_some());

        let p1 = self.p4_mut()
            .next_table_mut(page.p4_index())
            .and_then(|p3| p3.next_table_mut(page.p3_index()))
            .and_then(|p2| p2.next_table_mut(page.p2_index()))
            .expect("huge pages are not supported yet");

        let frame = p1[page.p1_index()].pointed_frame().unwrap();

        p1[page.p1_index()].set_unused();

        tlb::flush(NVirtualAddress(page.start_address()));

        // FIXME: free P{1,2,3} table if empty
        allocator.dealloc(frame);
    }
}

pub struct ActivePageTable {
    mapper: Mapper,
}

impl ActivePageTable {
    unsafe fn new() -> ActivePageTable {
        ActivePageTable { mapper: Mapper::new() }
    }

    pub fn with(&mut self, table: &mut InactivePageTable, tmp_page: &mut TmpPage, f: impl FnOnce(&mut Mapper)) {
        {
            // Backup current recursive mapping
            let old_rec = self.p4()[511];

            // Map temporary_page to current P4 table
            let p4_table = tmp_page.map_table_frame(old_rec.pointed_frame().unwrap(), self);

            // overwrite recursive mapping
            self.p4_mut()[511].set(table.p4_frame.clone(), F::PRESENT | F::WRITABLE);
            tlb::flush_all();

            // execute f in the new context
            f(self);

            // restore recursive mapping to original p4 table
            p4_table[511] = old_rec;
            tlb::flush_all();
        }

        tmp_page.unmap(self);
    }

    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        let old_table = InactivePageTable {
            p4_frame: Frame::containing_address(cr3().0 as usize),
        };

        unsafe {
            cr3_write(NPhysicalAddress(new_table.p4_frame.start_address() as u64));
        }

        old_table
    }
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Self::Target {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mapper
    }
}

pub struct InactivePageTable {
    p4_frame: Frame,
}

impl InactivePageTable {
    pub fn new(frame: Frame, active_table: &mut ActivePageTable, tmp_page: &mut TmpPage) -> InactivePageTable {
        {
            let table = tmp_page.map_table_frame(frame.clone(), active_table);
            table.clear();
            // Set up recursive mapping
            table[511].set(frame.clone(), F::PRESENT | F::WRITABLE);
        }
        tmp_page.unmap(active_table);

        InactivePageTable { p4_frame: frame }
    }
}

pub struct TmpPage {
    page: Page,
    allocator: TmpPageFrameAlloc,
}

impl TmpPage {
    pub fn new(page: Page, allocator: &mut impl FrameAlloc) -> TmpPage {
        TmpPage { page, allocator: TmpPageFrameAlloc::new(allocator) }
    }

    /// Maps the temporary page to the given frame in the active table.
    /// Returns the start address of the temporary page.
    pub fn map(&mut self, frame: Frame, active_table: &mut ActivePageTable) -> VirtualAddress {
        assert!(active_table.translate_page(self.page).is_none(), "temporary page is already mapped");
        active_table.map_to(self.page, frame, F::WRITABLE, &mut self.allocator);
        self.page.start_address()
    }

    /// Unmaps the temporary page in the active table.
    pub fn unmap(&mut self, active_table: &mut ActivePageTable) {
        active_table.unmap(self.page, &mut self.allocator)
    }

    /// Maps the temporary page to the given page table frame in the active
    /// table. Returns a reference to the now mapped table.
    pub fn map_table_frame(&mut self, frame: Frame, active_table: &mut ActivePageTable) -> &mut PageTable<L1> {
        unsafe { &mut *(self.map(frame, active_table) as *mut _) }
    }
}

struct TmpPageFrameAlloc([Option<Frame>; 3]);

impl TmpPageFrameAlloc {
    fn new(parent_allocator: &mut impl FrameAlloc) -> TmpPageFrameAlloc {
        TmpPageFrameAlloc([
            parent_allocator.alloc(),
            parent_allocator.alloc(),
            parent_allocator.alloc(),
        ])
    }
}

impl FrameAlloc for TmpPageFrameAlloc {
    fn alloc(&mut self) -> Option<Frame> {
        for frame_option in &mut self.0 {
            if frame_option.is_some() {
                return frame_option.take();
            }
        }
        None
    }

    fn dealloc(&mut self, frame: Frame) {
        for frame_option in &mut self.0 {
            if frame_option.is_none() {
                *frame_option = Some(frame);
                return;
            }
        }
        unreachable!();
    }
}

pub fn remap_kernel(allocator: &mut impl FrameAlloc, boot_info: &BootInformation) {
    let mut tmp_page = TmpPage::new(Page { number: REMAP_TMP_PAGE_NUMBER }, allocator);

    let mut active_table = unsafe {ActivePageTable::new()};

    let mut new_table = {
        let frame = allocator.alloc().expect("out of memory");
        InactivePageTable::new(frame, &mut active_table, &mut tmp_page)
    };

    active_table.with(&mut new_table, &mut tmp_page, |mapper| {
        // Identity map kernel sections
        let elf_sections_tag = boot_info.elf_sections_tag().expect("Elf sections tag required");

        for section in elf_sections_tag.sections() {
            if !section.is_allocated() {
                // Skip not allocated sections
                continue;
            }

            assert_eq!(section.start_address() % PAGE_SIZE, 0, "kernel sections need to be page aligned");

            let flags = EntryFlags::from_elf_section_flags(section);

            let start_frame = Frame::containing_address(section.start_address());
            let end_frame = Frame::containing_address(section.end_address() - 1);
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                mapper.identity_map(frame, flags, allocator);
            }
        }

        // Identity map VGA text buffer
        let vga_buffer_frame = Frame::containing_address(VGA_TEXT_BUFFER_ADDR);
        mapper.identity_map(vga_buffer_frame, F::WRITABLE, allocator);

        // Identity map Multiboot info
        let multiboot_start = Frame::containing_address(boot_info.start_address());
        let multiboot_end = Frame::containing_address(boot_info.end_address() - 1);
        for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
            mapper.identity_map(frame, F::PRESENT, allocator);
        }
    });

    let old_table = active_table.switch(new_table);

    let old_p4_page = Page::containing_address(
        old_table.p4_frame.start_address()
    );

    active_table.unmap(old_p4_page, allocator);

    kprintln!("remaped kernel successfully");
}
