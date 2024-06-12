/* implemention of interrupt descriptor table */

mod pic8259;

use spin;
use crate::gdt;
use crate::print;
use crate::println;
use crate::hlt_loop;
use pic8259::ChainedPics;
use lazy_static::lazy_static;
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> = 
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

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


// create a Interrupt Descriptor Table
// lazy_static make it have a long lifetime and can be modified
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
            // In some situation such as stack overflow, need switch the stack to run normally
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX); // when call stack by name maybe induce fault
        }
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

// initalize the new IDT
pub fn init_idt() {
    IDT.load();
}

// CPU will start receiving timer interrupts when enable interrupt
// so we need a timer interrupt
extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame)
{
    print!(".");

    // PIC except an explicit "end of interrupt" signal after the interrupt was processed
    // so we need a reply
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

// when press a key, the keyboard controller will send a interrupt
// so set keyboard handler function
extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame)
{
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60); // PS/2 controller
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }

}

// set page fault exception
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read()); // CR2 register store the virtual address that caused the page fault
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

// set double fault exception
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

// set breakpoint exception
extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame)
{
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}
