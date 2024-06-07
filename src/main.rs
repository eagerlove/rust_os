// Disable std and main
#![no_std]
#![no_main]

// implement a custom test framework
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]

// make test_runner call test_main instead of main
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use blog_os::print;
use blog_os::println;
use blog_os::task::{Task, simple_executor::SimpleExecutor};
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

// set kernel_main function as entry
entry_point!(kernel_main);

// below is the example_task function again so that you don't have to scroll up
async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

fn kernel_main(_boot_info: &'static BootInfo) -> ! {

    println!("Hell♂ W♀rld{}", "!");
    blog_os::init();

    let mut executor = SimpleExecutor::new();
    executor.spawn(Task::new(example_task()));
    executor.run();

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    blog_os::hlt_loop();

}

// This function is called on panic
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    blog_os::hlt_loop();
}

// panic for test
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}



