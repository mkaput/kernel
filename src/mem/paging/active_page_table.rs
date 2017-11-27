use core::ops::{Deref, DerefMut};

use x86_64::instructions::tlb;
use x86_64::registers::control_regs::{cr3, cr3_write};
use x86_64::PhysicalAddress as NPhysicalAddress;

use super::frame::Frame;
use super::inactive_page_table::InactivePageTable;
use super::mapper::Mapper;
use super::page_table::EntryFlags as F;
use super::tmp_page::TmpPage;

pub struct ActivePageTable {
    mapper: Mapper,
}

impl ActivePageTable {
    pub unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            mapper: Mapper::new(),
        }
    }

    pub fn with(
        &mut self,
        table: &mut InactivePageTable,
        tmp_page: &mut TmpPage,
        f: impl FnOnce(&mut Mapper),
    ) {
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
