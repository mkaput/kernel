use super::active_page_table::ActivePageTable;
use super::frame::Frame;
use super::page_table::EntryFlags as F;
use super::tmp_page::TmpPage;

pub struct InactivePageTable {
    pub p4_frame: Frame,
}

impl InactivePageTable {
    pub fn new(
        frame: Frame,
        active_table: &mut ActivePageTable,
        tmp_page: &mut TmpPage,
    ) -> InactivePageTable {
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
