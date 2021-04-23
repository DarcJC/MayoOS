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

    use mayoos::memory;
    use x86_64::VirtAddr;

    println!("Welcome to Mayo OS!");

    mayoos::init();
    
    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut page_table = unsafe { memory::init(physical_memory_offset) };
    let mut frame_allocator = unsafe {
        mayoos::memory::BootInfoFrameAllocator::new(&boot_info.memory_map)
    };

    mayoos::memory::allocator::init_heap(&mut page_table, &mut frame_allocator)
          .expect("heap initialization failed");

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

