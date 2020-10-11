#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(mayoos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use mayoos::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {

    println!("Welcome to Mayo OS!");

    mayoos::init();

    #[cfg(test)]
    test_main();

    println!("I'm still alive!");

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

