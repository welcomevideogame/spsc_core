use std::sync::{Arc, atomic::Ordering};

use crate::spsc::Inner;

#[derive(Debug)]
pub struct Receiver<T> {
    pub(super) inner: Arc<Inner<T>>,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Option<T> {
        let mut buffer_lock = self.inner.buffer.lock().unwrap();
        loop {
            if let Some(val) = buffer_lock.pop_front() {
                self.inner.condvar.notify_one();
                return Some(val);
            }
            buffer_lock = self.inner.condvar.wait(buffer_lock).unwrap();
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.inner.closed.store(true, Ordering::Release);
        self.inner.condvar.notify_one();
    }
}
