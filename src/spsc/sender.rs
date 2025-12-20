use std::sync::{Arc, atomic::Ordering};

use crate::spsc::{Inner, SendError};

#[derive(Debug)]
pub struct Sender<T> {
    pub(super) inner: Arc<Inner<T>>,
}

impl<T> Sender<T> {
    pub fn send(&self, item: T) -> Result<(), SendError<T>> {
        let mut buffer_lock = self.inner.buffer.lock().unwrap();
        loop {
            if buffer_lock.len() < self.inner.capacity {
                buffer_lock.push_back(item);
                break;
            }
            buffer_lock = match self.inner.condvar.wait(buffer_lock) {
                Ok(lock) => lock,
                Err(_) => return Err(SendError(item)),
            }
        }
        self.inner.condvar.notify_one();
        Ok(())
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.inner.closed.store(true, Ordering::Release);
    }
}
