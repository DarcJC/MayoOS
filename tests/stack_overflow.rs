
#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use lazy_static::lazy_static;
use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame,
};
use mayoos::{
    serial_println,
    serial_print, 
    exit_qemu_by_port, 
    QemuExitCode
};

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault.set_handler_fn(test_double_fault_handler)
                .set_stack_index(mayoos::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("Testing kernel stack overflow...\t");

    mayoos::gdt::init();
    init_test_idt();

    stack_overflow();

    panic!("[Failed: Execution continued after stack overflow]");
}

pub fn init_test_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu_by_port(QemuExitCode::Success);
    loop {}
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read();  // prevent tail recursion optimizations
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    mayoos::test_panic_handler(_info);
}



