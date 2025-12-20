use std::sync::{Arc, Mutex};

use crate::spsc::{Inner, SendError};

#[derive(Debug)]
pub struct Sender<T> {
    pub(super) inner: Arc<Mutex<Inner<T>>>,
}
impl<T> Sender<T> {
    pub fn send(&self, item: T) -> Result<(), SendError<T>> {
        loop {
            let lock = self.inner.lock();
        }
    }
}
