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
            let mut buffer_lock = self.inner.buffer.lock().await;
            if buffer_lock.len() < self.inner.capacity {
                buffer_lock.push_back(item);
                break;
            }
            drop(buffer_lock);
            self.inner.notify.notified().await;
        }
        self.inner.notify.notify_one();
        Ok(())
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.inner.closed.store(true, Ordering::Release);
        self.inner.notify.notify_one();
    }
}
