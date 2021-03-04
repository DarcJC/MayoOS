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

entry_point!(mayo_main);

fn mayo_main(boot_info: &'static BootInfo) -> ! {

    use mayoos::memory::active_level_4_table;
    use x86_64::VirtAddr;

    println!("Welcome to Mayo OS!");

    mayoos::init();
    
    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let l4_table = unsafe { active_level_4_table(physical_memory_offset) };

    for (i, entry) in l4_table.iter().enumerate() {
        if !entry.is_unused() {
            println!("L4 Entry {}: {:?}", i, entry);
        }
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

