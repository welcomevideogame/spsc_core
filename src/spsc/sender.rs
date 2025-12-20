use std::sync::{Arc, Mutex};

use crate::spsc::{Inner, SendError};

#[derive(Debug)]
pub struct Sender<T> {
    pub(super) inner: Arc<Mutex<Inner<T>>>,
}
impl<T> Sender<T> {
    pub fn send(&self, item: T) -> Result<(), SendError<T>> {
        loop {
            let mut lock = match self.inner.lock() {
                Ok(lock) => lock,
                Err(_) => return Err(SendError(item)),
            };
            if lock.is_closed() {
                return Err(SendError(item));
            }
            if lock.buffer.len() < lock.capacity {
                lock.buffer.push_back(item);
                return Ok(());
            }
        }
    }
}
