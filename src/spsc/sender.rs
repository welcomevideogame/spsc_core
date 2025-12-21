use std::sync::{Arc, atomic::Ordering};

use crate::spsc::{Inner, SendError};

#[derive(Debug)]
pub struct Sender<T> {
    pub(super) inner: Arc<Inner<T>>,
}

impl<T> Sender<T> {
    pub async fn send(&self, item: T) -> Result<(), SendError<T>> {
        loop {
            if self.inner.is_closed() {
                return Err(SendError(item));
            }

            let tail = self.inner.tail.load(Ordering::Relaxed);
            let head = self.inner.head.load(Ordering::Acquire);

            let next = (tail + 1) % self.inner.capacity;
            if next != head {
                unsafe {
                    (*self.inner.buffer[tail].get()).write(item);
                }
                self.inner.tail.store(next, Ordering::Release);
                self.inner.notify.notify_one();
                return Ok(());
            }
            self.inner.notify.notified().await;
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.inner.closed.store(true, Ordering::Release);
        self.inner.notify.notify_one();
    }
}
