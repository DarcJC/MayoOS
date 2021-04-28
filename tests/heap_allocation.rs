
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mayoos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{BootInfo, entry_point};
use mayoos::{halt_loop, memory::{allocator::{HEAP_SIZE, init_heap}, BootInfoFrameAllocator, init}};
use core::panic::PanicInfo;

extern crate alloc;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    use x86_64::VirtAddr;

    mayoos::init();

    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut page_table = unsafe { init(physical_memory_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::new(&boot_info.memory_map)
    };

    init_heap(&mut page_table, &mut frame_allocator)
             .expect("heap initialization failed");

    test_main();

    halt_loop()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    mayoos::test_panic_handler(info)
}

#[test_case]
fn simple_allocation() {
    use alloc::boxed::Box;
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}

#[test_case]
fn large_vec() {
    use alloc::vec::Vec;

    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes() {
    use alloc::boxed::Box;

    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

#[test_case]
fn many_boxes_long_lived() {
    use alloc::boxed::Box;

    let long_lived = Box::new(1); // new
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long_lived, 1); // new
}
