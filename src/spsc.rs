#![allow(dead_code, unused_variables)]

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

pub struct Queue<T> {
    buffer: VecDeque<T>,
    capacity: usize,
}

impl<T> Queue<T> {
    pub fn new(capacity: usize) -> (Sender<T>, Receiver<T>) {
        let vd = VecDeque::with_capacity(capacity);
        let queue = Queue {
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

pub struct Sender<T> {
    inner: Arc<Mutex<Queue<T>>>,
}

impl<T> Sender<T> {
    pub fn send(&self, item: T) {
        todo!()
    }
}

pub struct Receiver<T> {
    inner: Arc<Mutex<Queue<T>>>,
}

impl<T> Sender<T> {
    pub fn recv(&self) -> Option<T> {
        todo!()
    }
}
