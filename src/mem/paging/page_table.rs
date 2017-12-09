//! Page tables

use core::ops::{Index, IndexMut};
use core::marker::PhantomData;

use multiboot2::ElfSection;

use super::{Frame, FrameAlloc};

use self::EntryFlags as F;

// We have mapped this address to P4 table in our boot32 code.
const P4_ADDRESS: usize = 0xffff_ffff_ffff_f000;

const ENTRY_ADDR_MASK: usize = 0x000f_ffff_ffff_f000;

pub const ENTRY_COUNT: usize = 512;

/// Pointer to P4 (Page-Map Level-4 Table)
pub const P4: *mut PageTable<L4> = P4_ADDRESS as *mut _;

pub trait TableLevel {}

pub enum L4 {}

pub enum L3 {}

pub enum L2 {}

pub enum L1 {}

impl TableLevel for L4 {}

impl TableLevel for L3 {}

impl TableLevel for L2 {}

impl TableLevel for L1 {}

pub trait HierarchicalLevel: TableLevel {
    type NextLevel: TableLevel;
}

impl HierarchicalLevel for L4 {
    type NextLevel = L3;
}

impl HierarchicalLevel for L3 {
    type NextLevel = L2;
}

impl HierarchicalLevel for L2 {
    type NextLevel = L1;
}

/// Single page table
pub struct PageTable<L: TableLevel> {
    entries: [Entry; ENTRY_COUNT],
    level: PhantomData<L>,
}

impl<L: TableLevel> PageTable<L> {
    pub fn clear(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }
}

impl<L: HierarchicalLevel> PageTable<L> {
    fn next_table_address(&self, index: usize) -> Option<usize> {
        let entry_flags = self[index].flags();
        // FIXME: Why are we failing on huge pages?
        if entry_flags.contains(F::PRESENT) && !entry_flags.contains(F::HUGE_PAGE) {
            let table_address = self as *const _ as usize;
            Some(make_address_canonical((table_address << 9) | (index << 12)))
        } else {
            None
        }
    }

    pub fn next_table(&self, index: usize) -> Option<&PageTable<L::NextLevel>> {
        self.next_table_address(index)
            .map(|address| unsafe { &*(address as *const _) })
    }

    pub fn next_table_mut(&mut self, index: usize) -> Option<&mut PageTable<L::NextLevel>> {
        self.next_table_address(index)
            .map(|address| unsafe { &mut *(address as *mut _) })
    }

    pub fn next_table_create(
        &mut self,
        index: usize,
        allocator: &mut impl FrameAlloc,
    ) -> &mut PageTable<L::NextLevel> {
        if self.next_table(index).is_none() {
            assert!(
                !self.entries[index].flags().contains(F::HUGE_PAGE),
                "huge pages are not supported yet"
            );
            let frame = allocator.alloc().expect("out of memory");
            self.entries[index].set(frame, F::PRESENT | F::WRITABLE);
            self.next_table_mut(index).unwrap().clear();
        }
        self.next_table_mut(index).unwrap()
    }
}

impl<L: TableLevel> Index<usize> for PageTable<L> {
    type Output = Entry;

    fn index(&self, index: usize) -> &Entry {
        &self.entries[index]
    }
}

impl<L: TableLevel> IndexMut<usize> for PageTable<L> {
    fn index_mut(&mut self, index: usize) -> &mut Entry {
        &mut self.entries[index]
    }
}

/// Single page table entry
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Entry(u64);

impl Entry {
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    pub fn pointed_frame(&self) -> Option<Frame> {
        if self.flags().contains(F::PRESENT) {
            Some(Frame::containing_address(self.0 as usize & ENTRY_ADDR_MASK))
        } else {
            None
        }
    }

    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    /// Set value of this entry.
    ///
    /// The start address of a frame should be page aligned and smaller than 2^52
    /// (since x86 uses 52bit physical addresses). This function asserts that this
    /// condition holds.
    pub fn set(&mut self, frame: Frame, flags: EntryFlags) {
        assert_eq!(frame.start_address() & !ENTRY_ADDR_MASK, 0);
        self.0 = (frame.start_address() as u64) | flags.bits();
    }
}

bitflags! {
    // Bits 9-11 and 52-62 can be freely used by OS
    /// Page table entry flags
    pub struct EntryFlags: u64 {
        /// Page is currently in memory.
        const PRESENT =         1 << 0;
        /// Writing to this page is allowed.
        const WRITABLE =        1 << 1;
        /// Page is accessible from user mode.
        const USER_ACCESSIBLE = 1 << 2;
        /// Writes go directly to memory.
        const WRITE_THROUGH =   1 << 3;
        /// No cache is used for this page.
        const NO_CACHE =        1 << 4;
        /// CPU sets this bit when this page is used.
        const ACCESSED =        1 << 5;
        /// CPU sets this bit when this page is being written.
        const DIRTY =           1 << 6;
        /// Must be 0 in P1 and P4, creates a 1GiB page in P3, creates a 2MiB page in P2.
        const HUGE_PAGE =       1 << 7;
        /// Page isn't flushed from caches on address space switch.
        const GLOBAL =          1 << 8;
        /// Forbid executing code on this page (the NXE bit in the EFER register must be set).
        const NO_EXECUTE =      1 << 63;
    }
}

impl EntryFlags {
    pub fn from_elf_section_flags(section: &ElfSection) -> EntryFlags {
        use multiboot2::{ELF_SECTION_ALLOCATED, ELF_SECTION_EXECUTABLE, ELF_SECTION_WRITABLE};

        let mut flags = EntryFlags::empty();

        if section.flags().contains(ELF_SECTION_ALLOCATED) {
            // section is loaded to memory
            flags = flags | F::PRESENT;
        }

        if section.flags().contains(ELF_SECTION_WRITABLE) {
            flags = flags | F::WRITABLE;
        }

        if !section.flags().contains(ELF_SECTION_EXECUTABLE) {
            flags = flags | F::NO_EXECUTE;
        }

        flags
    }
}

/// Addresses are expected to be canonical (bits 48-63 must be the same as bit 47),
/// otherwise the CPU will #GP when we ask it to translate it.
fn make_address_canonical(address: usize) -> usize {
    let sign_extension = 0o177777_000_000_000_000_0000 * ((address >> 47) & 0b1);
    (address & ((1 << 48) - 1)) | sign_extension
}
