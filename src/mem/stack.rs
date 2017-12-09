use super::paging::{ActivePageTable, FrameAlloc, Page, PageIter};
use super::paging::EntryFlags as F;

#[derive(Debug)]
pub struct Stack {
    pub top: usize,
    pub bottom: usize,
    __guard: (),
}

impl Stack {
    fn new(top: usize, bottom: usize) -> Stack {
        assert!(top > bottom);
        Stack {
            top,
            bottom,
            __guard: (),
        }
    }
}

#[derive(Debug)]
pub struct StackAllocator {
    page_range: PageIter,
}

impl StackAllocator {
    pub const fn new(page_range: PageIter) -> StackAllocator {
        StackAllocator { page_range }
    }

    pub fn alloc(
        &mut self,
        active_table: &mut ActivePageTable,
        frame_alloc: &mut impl FrameAlloc,
        size_in_pages: usize,
    ) -> Option<Stack> {
        if size_in_pages == 0 {
            return None;
        }

        let mut range = self.page_range.clone();

        let guard_page = range.next();
        let start_page = range.next();
        let end_page = if size_in_pages == 1 {
            start_page
        } else {
            range.nth(size_in_pages - 2)
        };

        if let (Some(_), Some(start), Some(end)) = (guard_page, start_page, end_page) {
            for page in Page::range_inclusive(start, end) {
                active_table.map(page, F::WRITABLE, frame_alloc);
            }

            self.page_range = range;

            Some(Stack::new(end.end_address(), start.start_address()))
        } else {
            None
        }
    }
}
