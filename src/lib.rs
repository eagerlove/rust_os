/* function library of kenerl */

#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

pub mod gdt;
pub mod task;
pub mod serial;
pub mod memory;
pub mod allocator;
pub mod vga_buffer;
pub mod interrupts;
use core::panic::PanicInfo;


// port-mapped I/O to exit qemu
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10, // wirte 0 port (0 << 1) | 1
    Failed = 0x11,  // wirte 1 port (1 << 1) | 1
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4); // 0xf4 is a generally unused port on x86's IO bus
        port.write(exit_code as u32); // iosize: 4 bytes
    }
}

// automatically printing log template for all test which imeplement run()
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>()); // print function path
        self();
        serial_println!("[ok]");
    }
}

// halt CPU before next interrupt occur for decreasing the CPU usage
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

pub fn init() {
    // load GDT and IDT to kernel
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    // enable interrupt
    x86_64::instructions::interrupts::enable();

}

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

// Entry point for "cargo test"
#[cfg(test)] // only for test
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    // lib.rs is independtly of the main.rs
    // so add it when running test lib
    init();
    test_main();
    hlt_loop();

}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}


