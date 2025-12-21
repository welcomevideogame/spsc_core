use std::{
    future::poll_fn,
    pin::Pin,
    sync::{Arc, atomic::Ordering},
    task::Poll,
};

use futures::Stream;

use crate::spsc::Inner;

#[derive(Debug)]
pub struct Receiver<T> {
    pub(super) inner: Arc<Inner<T>>,
}

impl<T> Receiver<T> {
    pub async fn recv(&self) -> Option<T> {
        loop {
            let head = self.inner.head.load(Ordering::Relaxed);
            let tail = self.inner.tail.load(Ordering::Acquire);

            if head == tail {
                if self.inner.closed.load(Ordering::Relaxed) {
                    return None;
                }

                poll_fn(|cx| {
                    self.inner.waker.register(cx.waker());
                    let head = self.inner.head.load(Ordering::Relaxed);
                    let tail = self.inner.tail.load(Ordering::Acquire);
                    if head != tail {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                })
                .await;

                continue;
            }

            // Safe because consumer has exclusive access to this slot
            let value = unsafe { (*self.inner.buffer[head].get()).assume_init_read() };
            self.inner
                .head
                .store((head + 1) % self.inner.capacity, Ordering::Release);

            self.inner.waker.wake();
            return Some(value);
        }
    }
}

pub struct ReceiverStream<'a, T> {
    receiver: &'a Receiver<T>,
    pending: Option<Pin<Box<dyn Future<Output = Option<T>> + Send + 'a>>>,
}

impl<'a, T> ReceiverStream<'a, T> {
    pub fn new(rx: &'a Receiver<T>) -> Self {
        Self {
            receiver: rx,
            pending: None,
        }
    }
}

impl<'a, T> Stream for ReceiverStream<'a, T>
where
    T: Send + Sync + 'a,
{
    type Item = T;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if self.pending.is_none() {
            let fut = self.receiver.recv();
            self.pending = Some(Box::pin(fut));
        }

        let fut = self.pending.as_mut().unwrap();
        match fut.as_mut().poll(cx) {
            Poll::Ready(opt) => {
                self.pending = None;
                Poll::Ready(opt)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.inner.closed.store(true, Ordering::Release);
        self.inner.waker.wake();
    }
}
