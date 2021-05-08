use x86_64::instructions::port::{PortWriteOnly, PortReadOnly};
use alloc::vec::Vec;

/// PCI CONFIG_ADDRESS
///
/// `0x8000_0000 | bus << 16 | device << 11 | function << 8 | offset`
pub struct PCIConfigAddr(u32);

pub type Word = u16;
pub type DWord = u32;

impl PCIConfigAddr {
    /// Create a new *PCI config address* with overflow check.
    ///
    /// Only bits lower than field length will be passed.
    pub fn new(
        bus_number: u8,
        device_number: u8,
        func_number: u8,
        offset: u8,
    ) -> Self {
        let bus_number = (bus_number as u32);
        let device_number = (device_number as u32) & 0b0001_1111;
        let func_number = (func_number as u32) & 0b0000_0111;
        let offset = (offset as u32) & 0b1111_1100;
        let mut tmp =
            0x8000_0000
                | bus_number << 16
                | device_number << 11
                | func_number << 8
                | offset;
        PCIConfigAddr(tmp)
    }

    /// Same as `new` but without overflow check
    pub fn new_unchecked(
        bus_number: u8,
        device_number: u8,
        func_number: u8,
        offset: u8,
    ) -> Self {
        let mut tmp =
            0x8000_0000
                | (bus_number as u32) << 16
                | (device_number as u32) << 11
                | (func_number as u32) << 8
                | (offset as u32);
        PCIConfigAddr(tmp)
    }

    /// Writing the address to 0xCF8 port.
    pub fn write_to_port(&self) {
        let data = self.0;
        let mut port: PortWriteOnly<u32> = PortWriteOnly::new(0xCF8);
        unsafe {
            port.write(data);
        }
    }
}

/// Read port 0xCF8 's value
pub fn read_current_config_addr() -> u32 {
    let mut port: PortReadOnly<u32> = PortReadOnly::new(0xCF8);
    unsafe {
        port.read()
    }
}

/// Read port 0xCFC as word
///
/// ## Example
/// ```
/// # use mayoos::utils::pci::{PCIConfigAddr, read_config_data_word};
/// let addr = PCIConfigAddr::new(0, 0, 0, 0);
/// let vendor = read_config_data_word(0);
/// ```
pub fn read_config_data_word(offset: u8) -> u16 {
    let mut port: PortReadOnly<u32> = PortReadOnly::new(0xCFC);
    unsafe {
        (port.read() >> ((offset as u16 & 2) * 8)) as u16
    }
}

/// Read vendor id and device id using given bus and slot
///
/// **Don't use this function in async control stream**
fn get_device_vendor_and_id(bus: u8, device: u8) -> (Word, Word) {
    let addr = PCIConfigAddr::new(bus, device, 0, 0);
    addr.write_to_port();
    let res = read_config_data_word(0);
    (res, read_config_data_word(0b10))
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    bus: u8,
    slot: u8,
    id: Word,
    vendor: Word,
}

impl DeviceInfo {
    pub const fn new(bus: u8, slot: u8, id: Word, vendor: Word) -> Self {
        Self {
            bus,
            slot,
            id,
            vendor,
        }
    }
}

/// Check if the device exist
///
/// Return true if device exist
fn check_device(bus: u8, device: u8) -> bool {
    get_device_vendor_and_id(bus, device).0 != 0xFFFF
}

/// Discover all devices on the bus
pub fn scan_bus_devices(bus: u8) -> Vec<DeviceInfo> {
    let mut res: Vec<DeviceInfo> = Vec::with_capacity(32);

    for device_id in 0..32 {
        if check_device(bus, device_id) {
            let info = get_device_vendor_and_id(bus, device_id);
            res.push(DeviceInfo::new(bus, device_id, info.1, info.0));
        }
    }

    res
}
