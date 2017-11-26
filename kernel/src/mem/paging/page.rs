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
}
