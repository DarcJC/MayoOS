
use x86_64::{
    structures::paging::PageTable,
    VirtAddr,
    PhysAddr,
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

pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr)
    -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_memory_offset)
}

#[deprecated]
fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr)
    -> Option<PhysAddr> {
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];
    let mut frame = level_4_table_frame;

    for &i in &table_indexes {
        // note that {PageTableIndex} is an u16

        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe {&*table_ptr};

        let entry = &table[i];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("Huge pages are not supported for now"),
        };
    }

    Some(frame.start_address() + u64::from(addr.page_offset()))
}
