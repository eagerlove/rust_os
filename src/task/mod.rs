use core::{future::Future, pin::Pin};
use core::task::{Context, Poll};
use alloc::boxed::Box;

pub mod simple_executor;
pub mod keyboard;

pub struct Task {
    future: Pin<Box<dyn Future< Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            future: Box::pin(future),
        }
    }

    // poll the stored furture
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}