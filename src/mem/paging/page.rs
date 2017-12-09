use core::ops::{Add, Sub};

use super::{PhysicalAddress, VirtualAddress, PAGE_SIZE};

/// Represents virtual memory page
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Page {
    pub(super) number: usize,
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

    /// Returns first virtual address after page end
    pub const fn end_address(self) -> PhysicalAddress {
        (self.number + 1) * PAGE_SIZE
    }

    pub(super) fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }

    pub(super) fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }

    pub(super) fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }

    pub(super) fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }

    pub fn range_inclusive(start: Page, end: Page) -> PageIter {
        PageIter { start, end }
    }
}

impl Add<usize> for Page {
    type Output = Page;

    fn add(self, rhs: usize) -> Page {
        Page {
            number: self.number + rhs,
        }
    }
}

impl Sub<usize> for Page {
    type Output = Page;

    fn sub(self, rhs: usize) -> Page {
        Page {
            number: self.number - rhs,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PageIter {
    start: Page,
    end: Page,
}

impl Iterator for PageIter {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
        if self.start <= self.end {
            let page = self.start;
            self.start.number += 1;
            Some(page)
        } else {
            None
        }
    }
}
