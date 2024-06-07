/* fixed size block allocation: offer different size block to allocate heap */

use super::Locked;
use alloc::alloc::GlobalAlloc;
use core::{alloc::Layout, mem, ptr::{self, NonNull}};

struct ListNode {
    next: Option<&'static mut ListNode>,
}

// The block sizes to use.
// The sizes must each be power of 2 because they are also used as
// the block alignment (alignments must be always powers of 2).
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

pub struct FixedSizeBlockAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap, // large allocations (>2 KB) 
}

// Choose an appropriate block size for the given layout
// Return: an index into the 'BLOCK_SIZES' array
fn list_index(layout: &Layout) -> Option<usize> {
    // align layout
    let required_block_size = layout.size().max(layout.align());
    // find the index of the first block that is at least as large as the required_block_size
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

impl FixedSizeBlockAllocator {
    // creates an empty FixedSizeBlockAllocator.
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    // initialize allocator
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size)
    }

    // allocates large block by using the fallback allocator
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}


// Global Allocator implementation
unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // get mutable reference
        let mut allocator = self.lock();
        // calculate the appropriate block size corresponding index
        match list_index(&layout) {
            Some(index) => {
                match allocator.list_heads[index].take() {
                    Some(node) => {
                        // find empty block by itering
                        allocator.list_heads[index] = node.next.take();
                        node as *mut ListNode as *mut u8
                    }
                    None => {
                        // no block exists in list => allocate new block
                        let block_size = BLOCK_SIZES[index];
                        // only works if all block sizes are power of 2
                        let block_align = block_size;
                        let layout = Layout::from_size_align(block_size, block_align)
                            .unwrap();
                        allocator.fallback_alloc(layout)
                    }
                }
            }
            // no block size fits for the allocation
            // use fallback allocator
            None => allocator.fallback_alloc(layout),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();

        match list_index(&layout) {
            Some(index) => {
                let new_node = ListNode {
                    next: allocator.list_heads[index].take(),
                };
                // verify that block has size and alignment required for storing value
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);
                let new_node_ptr = ptr as *mut ListNode;
                // overwirte the block
                new_node_ptr.write(new_node);
                // transform the new_node_ptr from None to mut*
                allocator.list_heads[index] = Some(&mut *new_node_ptr);
            }
            // no fitting block size exists
            // so use the deallocate method of fallback
            None => {
                let ptr = NonNull::new(ptr).unwrap();
                allocator.fallback_allocator.deallocate(ptr, layout);
            }
        }
    }
}