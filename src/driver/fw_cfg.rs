extern crate alloc;

use x86_64::instructions::port::{PortWriteOnly, PortReadOnly};
use alloc::string::String;
use spin::{Mutex};
use core::intrinsics::size_of;
use x86_64::VirtAddr;
use crate::memory::translate_addr;
use crate::println;
use crate::driver::{ByteOrderConverter, write_to_port, get_physical_memory_offset};
use core::iter::FromIterator;
use alloc::vec::Vec;

type FwCfgPort = u16;

pub const SELECTOR_PORT: FwCfgPort = 0x510;
pub const DATA_PORT: FwCfgPort = 0x511;
pub const DMA_PORT: FwCfgPort = 0x514;

/// set SELECTOR port to given selector value
///
/// unsafe because it might lead to condition race
pub unsafe fn set_selector_port(selector: u16) {
    let mut p = PortWriteOnly::new(SELECTOR_PORT);
    p.write(selector)
}

/// Reading data from DATA port.
///
/// Unsafe because calling the function while wrong selector value set
/// might have undefined behavior.
pub unsafe fn read_data() -> u8 {
    let mut p = PortReadOnly::new(DATA_PORT);
    p.read()
}

/// Check if using qemu
pub fn is_qemu() -> bool {
    unsafe {
        set_selector_port(0);
        let mut buf = [0x0; 4];
        for i in 0..4 {
            buf[i] = read_data();
        }
        String::from_utf8_lossy(&buf).eq("QEMU")
    }
}

type DMAControlCommand = u32;

pub const DMA_ERROR: DMAControlCommand = 0x01;
pub const DMA_READ: DMAControlCommand = 0x02;
pub const DMA_SKIP: DMAControlCommand = 0x04;
pub const DMA_SELECT: DMAControlCommand = 0x08;
pub const DMA_WRITE: DMAControlCommand = 0x0F;

#[repr(C)]
#[derive(Debug)]
pub struct DMAAccess {
    control: u32,
    length: u32,
    address: u64,
}

impl DMAAccess {
    const fn new() -> Self {
        return Self {
            control: 0,
            length: 0,
            address: 0,
        };
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FWCfgFile {
    size: u32,
    select: u16,
    reserved: u16,
    name: [u8; 56],
}

impl FWCfgFile {
    #![allow(dead_code)]
    pub fn get_name(&self) -> String {
        let char_array = self.name.map( |b| char::from(b));
        let char_array = char_array.iter().filter(|c| **c != '\0');
        String::from_iter(char_array)
    }
}

impl Default for FWCfgFile {
    fn default() -> Self {
        Self {
            size: 0,
            select: 0,
            reserved: 0,
            name: [0; 56],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FWCfgFiles {
    count: u32,
}

pub static DMA_CTL: Mutex<DMAAccess> = Mutex::new(DMAAccess::new());

unsafe fn to_dma_address(ptr: u64) -> u64 {
    translate_addr(
        VirtAddr::new(ptr), get_physical_memory_offset()
    ).unwrap().as_u64().swab()
}

unsafe fn dma_read_bytes(dest: u64, length: u32) {
    let mut dma = DMA_CTL.lock();
    let addr = translate_addr(VirtAddr::new((&*dma as *const DMAAccess) as u64), get_physical_memory_offset()).unwrap().as_u64();
    dma.address = to_dma_address(dest);
    dma.length = length.swab();
    dma.control = DMA_READ.swab();
    let high = ((addr >> 32) as u32).swab();
    let low = ((addr & 0xFFFFFFFF) as u32).swab();
    write_to_port(DMA_PORT, high);
    write_to_port(DMA_PORT + 4, low);
}

const _FD_SIZE: usize = size_of::<FWCfgFile>();

pub fn get_files() -> Vec<FWCfgFile> {
    let count: u32 = 0;
    let count_ptr: *const u32 = &count;
    unsafe {
        set_selector_port(0x0019);
        dma_read_bytes(count_ptr as u64, 4);
    }
    let count = count.swab();
    let mut res: Vec<FWCfgFile> = Vec::new();
    for _ in 0..count {
        let file = FWCfgFile::default();
        unsafe {
            dma_read_bytes((&file as *const FWCfgFile) as u64, _FD_SIZE as u32);
        }
        res.push(file.clone())
    }
    res
}

pub fn read_file_by_name(name: &str) -> Option<Vec<u8>> {
    let files = get_files();
    let mut res: Vec<u8> = Vec::new();
    let mut buf = 0u8;
    let buf_ptr= &buf as *const u8;
    if let Some(file) = files.iter().find(|f| {
        f.get_name() == name
    }) {
        println!("Reading file of size {} ... Please wait~", file.size.swab());
        unsafe {
            set_selector_port(file.select.swab());
            for _ in 0..file.size.swab() {
                dma_read_bytes(buf_ptr as u64, 1);
                res.push(buf);
            }
        }
        Some(res)
    } else {
        None
    }
}
