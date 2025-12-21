#![allow(dead_code)]

use std::mem::MaybeUninit;
use std::{
    cell::UnsafeCell,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
use tokio::sync::Notify;
mod error;
mod receiver;
mod sender;

#[derive(Debug)]
struct Inner<T> {
    buffer: Box<[UnsafeCell<MaybeUninit<T>>]>,
    capacity: usize,

    head: AtomicUsize, // consumer-owned
    tail: AtomicUsize, // producer-owned

    closed: AtomicBool,
    notify: Notify,
}
unsafe impl<T: Send> Send for Inner<T> {}
unsafe impl<T: Sync> Sync for Inner<T> {}

impl<T> Inner<T> {
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }
}

pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let buf = (0..capacity)
        .into_iter()
        .map(|_| UnsafeCell::new(MaybeUninit::uninit()))
        .collect::<Vec<_>>()
        .into_boxed_slice();

    let inner = Arc::new(Inner {
        buffer: buf,
        capacity,
        head: AtomicUsize::new(0),
        tail: AtomicUsize::new(0),
        closed: AtomicBool::new(false),
        notify: Notify::new(),
    });

    let sender = Sender {
        inner: Arc::clone(&inner),
    };

    let receiver = Receiver { inner };

    (sender, receiver)
}

pub use error::SendError;
pub use receiver::{Receiver, ReceiverStream};
pub use sender::Sender;
