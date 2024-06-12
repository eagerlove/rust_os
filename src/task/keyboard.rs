use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::{pin::Pin, task::{Poll, Context}};
use futures_util::stream:Stream;
use futures_util::task::AtomicWaker;

use crate::print;
use crate::println;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

// Called by the keyboard interrupt handler
// must not block or allocate
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () } // prevent construction of the struct from outside of the module
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("not initialized");

        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            // succeed: return the scancode wrapped
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            // fail: the queue is empty, wait
            None => Poll::Pending,
        }
    }
}
