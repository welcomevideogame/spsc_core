#![allow(dead_code, unused_variables)]

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct Channel<T> {
    buffer: VecDeque<T>,
    capacity: usize,
}

impl<T> Channel<T> {
    pub fn new(capacity: usize) -> (Sender<T>, Receiver<T>) {
        let vd = VecDeque::with_capacity(capacity);
        let queue = Channel {
            buffer: vd,
            capacity,
        };
        let inner = Arc::new(Mutex::new(queue));
        let sender = Sender {
            inner: inner.clone(),
        };
        let receiver = Receiver {
            inner: inner.clone(),
        };
        (sender, receiver)
    }
}

#[derive(Debug)]
pub struct Sender<T> {
    inner: Arc<Mutex<Channel<T>>>,
}

impl<T> Sender<T> {
    pub fn send(&self, item: T) {
        todo!()
    }
}

#[derive(Debug)]
pub struct Receiver<T> {
    inner: Arc<Mutex<Channel<T>>>,
}

impl<T> Sender<T> {
    pub fn recv(&self) -> Option<T> {
        todo!()
    }
}
