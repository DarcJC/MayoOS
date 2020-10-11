
use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame,
};
use pic8259_simple::ChainedPics;
use spin::Mutex;
use crate::{
    println,
    print,
};
use crate::gdt;
use lazy_static::lazy_static;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);

        idt
    };

    pub static ref PICS: Mutex<ChainedPics> = {
        Mutex::new(unsafe {
            ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
        })
    };
}


pub fn init_idt() {
    IDT.load();
}

// Exception interrupts
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    err_code: u64
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}\nERROR CODE: {}\n", stack_frame, err_code);
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: &mut InterruptStackFrame
) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}


// Hardware interrupts
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: &mut InterruptStackFrame
) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: &mut InterruptStackFrame
) {

    use x86_64::instructions::port::PortRead;
    use pc_keyboard::{
        layouts, DecodedKey, HandleControl,
        Keyboard, ScancodeSet1,
    };

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = {
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,
                HandleControl::Ignore))
        };
    }

    let mut keyboard = KEYBOARD.lock();
    let scancode: u8 = unsafe {
        PortRead::read_from_port(0x60)
    };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
