use std::sync::{Arc, Mutex};

use crate::spsc::Inner;

#[derive(Debug)]
pub struct Receiver<T> {
    pub(super) inner: Arc<Mutex<Inner<T>>>,
}
impl<T> Receiver<T> {
    pub fn recv(&self) -> Option<T> {
        loop {
            let mut lock = self.inner.lock().ok()?;
            if let Some(item) = lock.buffer.pop_front() {
                return Some(item);
            }
        }
    }
}
