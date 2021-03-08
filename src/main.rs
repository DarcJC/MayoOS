#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(mayoos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use mayoos::println;
use bootloader::{
    BootInfo,
    entry_point,
};
use x86_64::structures::paging::MapperAllSizes;

entry_point!(mayo_main);

fn mayo_main(boot_info: &'static BootInfo) -> ! {

    use mayoos::memory;
    use x86_64::VirtAddr;

    println!("Welcome to Mayo OS!");

    mayoos::init();
    
    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let page_table = unsafe { memory::init(physical_memory_offset) };

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = page_table.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }

    #[cfg(test)]
    test_main();

    println!("Mayo OS is now alive!");

    mayoos::halt_loop()
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);

    mayoos::halt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    mayoos::test_panic_handler(_info);
}

