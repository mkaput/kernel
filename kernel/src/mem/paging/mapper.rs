use core::ptr::Unique;

use x86_64::instructions::tlb;
use x86_64::VirtualAddress as NVirtualAddress;

use super::{PAGE_SIZE, VirtualAddress, PhysicalAddress};
use super::frame::Frame;
use super::frame_alloc::FrameAlloc;
use super::page::Page;
use super::page_table::{ENTRY_COUNT, EntryFlags, PageTable, L4, P4};
use super::page_table::EntryFlags as F;

pub struct Mapper {
    p4: Unique<PageTable<L4>>,
}

impl Mapper {
    pub unsafe fn new() -> Mapper {
        Mapper {
            p4: Unique::new_unchecked(P4),
        }
    }

    pub(super) fn p4(&self) -> &PageTable<L4> {
        unsafe { self.p4.as_ref() }
    }

    pub(super) fn p4_mut(&mut self) -> &mut PageTable<L4> {
        unsafe { self.p4.as_mut() }
    }

    /// Translates specified virtual address to physical address using currently used page table.
    /// Returns `None` if the address is not mapped.
    pub fn translate(&self, virtual_address: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = virtual_address % PAGE_SIZE;
        self.translate_page(Page::containing_address(virtual_address))
            .map(|frame| frame.start_address() + offset)
    }

    pub(super) fn translate_page(&self, page: Page) -> Option<Frame> {
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
