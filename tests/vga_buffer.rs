
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mayoos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[cfg(test)]
use mayoos::{serial_println, serial_print, println, vga_buffer};

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
    serial_print!("Testing println...\t");
    for i in 0..200 {
        println!("Test test test {}", i);
    }
    serial_println!("[ok]");
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    serial_print!("Testing println outputs...\t");

    let s = "Some test string that fits on a single line";

    let mut writer = vga_buffer::VGA_WRITER.lock();
    interrupts::without_interrupts(|| {
        writeln!(writer, "{}", s).unwrap();
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[vga_buffer::BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_char), c);
        }
    });

    serial_println!("[ok]");
}
