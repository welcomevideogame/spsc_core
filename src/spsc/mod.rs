//! # SPSC Async Channel
//!
//! This crate provides a **single-producer, single-consumer asynchronous channel**
//! implemented using a fixed-size ring buffer, atomic indices, and `AtomicWaker`.
//!
//! ## Features
//! - Lock-free send/receive operations.
//! - Async-friendly: `send` and `recv` return `Future`s.
//! - Properly wakes tasks only when needed, minimizing overhead.
//! - Closed-channel behavior supported.
//!
//! ## Safety
//! Internally uses `UnsafeCell<MaybeUninit<T>>` for storage. Safety is guaranteed
//! because only the producer ever writes to the tail slot, and only the consumer
//! ever reads from the head slot.
//!
//! ## Example
//! ```rust
//! use spsc::channel;
//! use tokio::spawn;
//!
//! let (tx, rx) = channel::<i32>(16);
//!
//! spawn(async move { tx.send(42).await.unwrap(); });
//! let value = rx.recv().await.unwrap();
//! assert_eq!(value, 42);
//! ```

use futures::task::AtomicWaker;
use std::mem::MaybeUninit;
use std::{
    cell::UnsafeCell,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
mod error;
mod receiver;
mod sender;

/// Internal structure holding the buffer and synchronization primitives
/// for a single-producer, single-consumer asynchronous channel.
///
/// # Overview
/// `Inner<T>` maintains a fixed-size ring buffer of type `T` along with atomic
/// indices to track the producer (`tail`) and consumer (`head`). It is designed
/// for **exactly one producer and one consumer**, allowing lock-free asynchronous
/// operations.
///
/// # Fields
/// - `buffer`: The fixed-size storage for channel items. Each slot is an
///   `UnsafeCell<MaybeUninit<T>>` to allow safe interior mutability while
///   avoiding uninitialized memory access.
/// - `capacity`: Total number of items the channel can hold.
/// - `head`: Index of the next item to read (consumer-owned).
/// - `tail`: Index of the next slot to write (producer-owned).
/// - `closed`: Atomic flag indicating whether the channel is closed.
/// - `waker`: AtomicWaker used to wake the waiting task when data becomes available.
///
/// # Safety
/// - `UnsafeCell` and `MaybeUninit` are safe here because:
///   1. Only the producer ever writes to `tail` slots.
///   2. Only the consumer ever reads from `head` slots.
/// - Requires `T: Send` to safely move items between tasks.
/// - The channel is **not safe for multiple producers or consumers**.
///
/// # Example
/// ```
/// use spsc::channel;
/// use tokio::spawn;
///
/// let (tx, rx) = channel::<i32>(16);
///
/// spawn(async move { tx.send(42).await.unwrap(); });
/// let value = rx.recv().await.unwrap();
/// assert_eq!(value, 42);
/// ```
#[derive(Debug)]
struct Inner<T> {
    buffer: Box<[UnsafeCell<MaybeUninit<T>>]>,
    capacity: usize,

    head: AtomicUsize, // consumer-owned
    tail: AtomicUsize, // producer-owned

    closed: AtomicBool,
    waker: AtomicWaker,
}

// Safety: single-producer writes to tail, single-consumer reads head
unsafe impl<T: Send> Send for Inner<T> {}
unsafe impl<T: Sync> Sync for Inner<T> {}

impl<T> Inner<T> {
    /// Returns `true` if channel has been closed.
    ///
    /// Once closed, the consumer will eventually get `None` when all items are consumed
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }
}

/// Creates a new single-producer, single-consumer asynchronous channel
/// with the specified `capacity`.
///
/// # Arguments
/// - `capacity`: Maximum number of items the channel can hold.
///
/// # Returns
/// A tuple `(Sender<T>, Receiver<T>)` representing the two halves of the channel.
///
/// # Behavior
/// - `Sender` can asynchronously send items into the channel.  
/// - `Receiver` can asynchronously receive items from the channel.  
/// - If the buffer is full, `send` will asynchronously wait until space becomes available.  
/// - If the buffer is empty, `recv` will asynchronously wait until an item is sent.  
/// - Closing the channel stops further sends and eventually causes `recv` to return `None`.
///
/// # Example
/// ```
/// use spsc::channel;
/// use tokio::spawn;
///
/// let (tx, rx) = channel::<i32>(8);
///
/// spawn(async move { tx.send(1).await.unwrap(); });
/// assert_eq!(rx.recv().await.unwrap(), 1);
/// ```
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
        waker: AtomicWaker::new(),
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
