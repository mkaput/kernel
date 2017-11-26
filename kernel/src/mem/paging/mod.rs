mod core_frame_alloc;
mod pt;

use core::ptr::Unique;

use x86_64::instructions::tlb;
use x86_64::VirtualAddress as NVirtualAddress;

use self::pt::*;
use self::pt::EntryFlags as F;

pub use self::core_frame_alloc::*;

pub type VirtualAddress = usize;
pub type PhysicalAddress = usize;

const PAGE_SIZE: usize = 4096;

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

    /// Returns frame end physical address.
    pub const fn end_address(&self) -> PhysicalAddress {
        (self.number + 1) * PAGE_SIZE - 1
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
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
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

    /// Returns page end virtual address.
    pub const fn end_address(&self) -> PhysicalAddress {
        (self.number + 1) * PAGE_SIZE - 1
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

pub struct ActivePageTable {
    p4: Unique<PageTable<L4>>,
}

impl ActivePageTable {
    pub unsafe fn new() -> ActivePageTable {
        ActivePageTable {
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

    pub fn map(&mut self, page: Page, flags: EntryFlags, allocator: &mut impl FrameAlloc) {
        let frame = allocator.alloc().expect("out of memory");
        self.map_to(page, frame, flags, allocator)
    }

    pub fn identity_map(
        &mut self,
        frame: Frame,
        flags: EntryFlags,
        allocator: &mut impl FrameAlloc,
    ) {
        let page = Page::containing_address(frame.start_address());
        self.map_to(page, frame, flags, allocator)
    }

    fn unmap(&mut self, page: Page, allocator: &mut impl FrameAlloc) {
        assert!(self.translate(page.start_address()).is_some());

        let p1 = self.p4_mut()
            .next_table_mut(page.p4_index())
            .and_then(|p3| p3.next_table_mut(page.p3_index()))
            .and_then(|p2| p2.next_table_mut(page.p2_index()))
            .expect("huge pages are not supported yet");

        let frame = p1[page.p1_index()].pointed_frame().unwrap();

        p1[page.p1_index()].set_unused();

        tlb::flush(NVirtualAddress(page.start_address()));

        // TODO free P{1,2,3} table if empty
        allocator.dealloc(frame);
    }
}
