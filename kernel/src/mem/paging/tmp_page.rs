use super::VirtualAddress;
use super::active_page_table::ActivePageTable;
use super::frame::Frame;
use super::frame_alloc::FrameAlloc;
use super::page::Page;
use super::page_table::{PageTable, L1};
use super::page_table::EntryFlags as F;

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
