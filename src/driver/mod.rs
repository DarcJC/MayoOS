use x86_64::instructions::port::{PortWrite, PortWriteOnly, PortRead, PortReadOnly};
use lazy_static::lazy_static;
use x86_64::VirtAddr;
use spin::RwLock;

pub mod fw_cfg;

pub unsafe fn write_to_port<T>
(addr: u16, value: T)
    where T: PortWrite {
    let mut p = PortWriteOnly::new(addr);
    p.write(value);
}

pub unsafe fn read_from_port<T>
(addr: u16) -> T
    where T: PortRead {
    let mut p: PortReadOnly<T> = PortReadOnly::new(addr);
    p.read()
}

pub trait ByteOrderConverter {
    fn swab(&self) -> Self;
}

impl ByteOrderConverter for u16 {
    fn swab(&self) -> Self {
        let x = *self;
        ((x & 0x00FFu16) << 8) | ((x & 0xFF00u16) >> 8)
    }
}

impl ByteOrderConverter for u32 {
    fn swab(&self) -> Self {
        let x = *self;
        ((x & 0x000000FFu32) << 24)
            | ((x & 0x0000FF00u32) << 8)
            | ((x & 0x00FF0000u32) >> 8)
            | ((x & 0xFF000000u32) >> 24)
    }
}

impl ByteOrderConverter for u64 {
    fn swab(&self) -> Self {
        let x = *self;
        ((x & 0x00000000000000FFu64) << 56)
            | ((x & 0x000000000000FF00u64) << 40)
            | ((x & 0x0000000000FF0000u64) << 24)
            | ((x & 0x00000000FF000000u64) << 8)
            | ((x & 0x000000FF00000000u64) >> 8)
            | ((x & 0x0000FF0000000000u64) >> 24)
            | ((x & 0x00FF000000000000u64) >> 40)
            | ((x & 0xFF00000000000000u64) >> 56)
    }
}

lazy_static!{
    static ref _PHYSICAL_MEMORY_OFFSET: RwLock<VirtAddr> = RwLock::new(VirtAddr::new(0));
}

pub fn init(phy_mem_offset: VirtAddr) {
    *(_PHYSICAL_MEMORY_OFFSET.write()) = phy_mem_offset;
}

pub fn get_physical_memory_offset() -> VirtAddr {
    return *_PHYSICAL_MEMORY_OFFSET.read()
}
