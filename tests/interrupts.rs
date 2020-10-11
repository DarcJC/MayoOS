
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mayoos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[cfg(test)]
use mayoos::{serial_println, serial_print};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    mayoos::init();
    test_main();

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    mayoos::test_panic_handler(_info);
}

#[test_case]
fn test_breakpoint_exception() {
    serial_print!("Testing breakpoint(INT3) exception... ");
    x86_64::instructions::interrupts::int3();
    serial_println!("[ok]");
}
