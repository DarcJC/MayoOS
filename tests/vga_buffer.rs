
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mayoos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[cfg(test)]
use mayoos::{serial_println, serial_print, println};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    mayoos::test_panic_handler(_info);
}

#[test_case]
fn test_println() {
    serial_print!("Testing println... ");
    for i in 0..200 {
        println!("Test test test {}", i);
    }
    serial_println!("[ok]");
}
