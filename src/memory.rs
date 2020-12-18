
use x86_64::{
    structures::paging::PageTable,
    VirtAddr,
};

pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) 
    -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (frame, _) = Cr3::read();

    let phys = frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}
