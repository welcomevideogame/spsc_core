use std::sync::{Arc, atomic::Ordering};

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

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.inner.closed.store(true, Ordering::Release);
        self.inner.notify.notify_one();
    }
}
