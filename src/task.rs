use core::pin::Pin;
use alloc::boxed::Box;
use core::future::Future;
use core::task::{Context, Poll};

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Self {
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

pub mod simple_executor;