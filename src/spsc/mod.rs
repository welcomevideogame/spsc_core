use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::{collections::VecDeque, sync::Condvar};
mod error;
mod receiver;
mod sender;

#[derive(Debug)]
struct Inner<T> {
    buffer: Mutex<VecDeque<T>>,
    capacity: usize,
    closed: AtomicBool,
    condvar: Condvar,
}

impl<T> Inner<T> {
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }
}

pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(Inner {
        buffer: Mutex::new(VecDeque::with_capacity(capacity)),
        capacity,
        closed: AtomicBool::new(false),
        condvar: Condvar::new(),
    });

    let sender = Sender {
        inner: Arc::clone(&inner),
    };

    let receiver = Receiver { inner };

    (sender, receiver)
}

pub use error::SendError;
pub use receiver::Receiver;
pub use sender::Sender;
