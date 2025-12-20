use std::{
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
            let mut buffer_lock = self.inner.buffer.lock().await;
            if buffer_lock.len() == 0 && self.inner.is_closed() {
                return None;
            }
            if let Some(val) = buffer_lock.pop_front() {
                self.inner.notify.notify_one();
                return Some(val);
            }
            drop(buffer_lock);
            self.inner.notify.notified().await;
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
    T: Send + 'a,
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
        self.inner.notify.notify_one();
    }
}
