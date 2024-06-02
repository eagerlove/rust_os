/* implementation of allocating heap memory */

mod bump;
mod linked_list;
mod fixed_size_block;

use core::ptr:: null_mut;
use alloc::alloc::{GlobalAlloc, Layout};
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB
    },
    VirtAddr,
};

// use bump::BumpAllocator;
// use linked_list::LinkedListAllocator;
use self::fixed_size_block::FixedSizeBlockAllocator;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100KB

#[global_allocator] // use the fllowing function as the global allocator
// Locked type: spinlock type for synchronization
// static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new()); 
// static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());


// map heap region to physical memory
// This function take mutable references to a Mapper and a FrameAllocator instance
// return value Result: unit type(success) or MapToError(fail)
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    // use heap start and size define page
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    // mapping for all heap pages
    for page in page_range {
        // allocate a physical frame for each heap page
        // Option::ok_or map to MapToError if error
        // ?: early to return
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        // allow heap page can be read and writen
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        // create the mapping in the active page table
        // success: return a MapperFlush instance
        // fail:    return error to the caller
        unsafe { 
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        };
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

// a wrapper around spin::Mutex to permit trait implementations.
// it can be used to wrap all kinds of types, not just allocators
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl <A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }
    // lock on the wrapped Mutex
    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

// align the given address 'addr' upwards to alignment 'align'
fn align_up(addr: usize, align: usize) -> usize {
    let remainder = addr % align;
    if remainder == 0 {
        addr // addr already aligned
    } else {
        addr - remainder + align
    }
    // (addr + align - 1) / align * align

    // only used when align is a power of 2 
    // (addr + align - 1) & !(align - 1)
}


// example allocator(not used)
pub struct Dummy;
unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called")
    }
}

