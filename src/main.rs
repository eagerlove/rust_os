// Disable std and main
#![no_std]
#![no_main]

// implement a custom test framework
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]

// make test_runner call test_main instead of main
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

use blog_os::allocator;
use blog_os::print;
use blog_os::println;
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

// set kernel_main function as entry
entry_point!(kernel_main);


fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use blog_os::memory::{self, BootInfoFrameAllocator};
    use x86_64:: VirtAddr;

    println!("Hell♂ W♀rld{}", "!");
    blog_os::init();

    // physical address turn to virtual address
    // boot_info.physical_memory_offset: 1649267441664 byte
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    
    // initialize a mapper
    let mut mapper = unsafe {
        memory::init(phys_mem_offset)
    };
    // create a frame allocator
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    // initialize heap
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");
    
    // allocate a number on the heap
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    // create a dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..10 {
        vec.push(i);
    }



    // print the underlying heap pointers for Box and Vec types
    println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    // clone 1 times will make reference count add 1
    let cloned_reference = reference_counted.clone();
    let cloned_reference_1 = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));
    core::mem::drop(cloned_reference);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference_1));
    
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



