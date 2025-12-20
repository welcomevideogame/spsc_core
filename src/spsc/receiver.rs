use std::sync::{Arc, Mutex};

use crate::spsc::Inner;

#[derive(Debug)]
pub struct Receiver<T> {
    pub(super) inner: Arc<Mutex<Inner<T>>>,
}
impl<T> Receiver<T> {
    pub fn recv(&self) -> Option<T> {
        todo!()
    }
}
