use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
mod error;
mod receiver;
mod sender;

#[derive(Debug)]
struct Inner<T> {
    buffer: VecDeque<T>,
    capacity: usize,
}

pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(Mutex::new(Inner {
        buffer: VecDeque::with_capacity(capacity),
        capacity,
    }));

    let sender = Sender {
        inner: Arc::clone(&inner),
    };

    let receiver = Receiver { inner };

    (sender, receiver)
}

pub use error::SendError;
pub use receiver::Receiver;
pub use sender::Sender;

