use std::{
    future::poll_fn,
    sync::{Arc, atomic::Ordering},
    task::Poll,
};

use crate::spsc::{Inner, SendError};

/// The sending half of a single-producer, single-consumer asynchronous channel.
///
/// `Sender<T>` allows the producer task to asynchronously send values into the channel.
///
/// # Behavior
/// - Only one `Sender` should exist per channel; cloning is possible but must be done carefully,
///   as the channel is fundamentally **single-producer**.
/// - If the buffer is full, `send` will asynchronously wait until space becomes available.
/// - If the channel is closed, `send` will return an error.
///
/// # Example
/// ```
/// use spsc::channel;
/// use tokio::spawn;
///
/// let (tx, rx) = channel::<i32>(16);
///
/// tokio::spawn(async move {
///     tx.send(42).await.unwrap();
/// });
/// ```
#[derive(Debug)]
pub struct Sender<T> {
    pub(super) inner: Arc<Inner<T>>,
}

impl<T> Sender<T> {
    /// Sends a value into the channel.
    ///
    /// If the channel is full, this method will asynchronously wait until space
    /// becomes available. Returns `Err(item)` if the channel has been closed.
    pub async fn send(&self, item: T) -> Result<(), SendError<T>> {
        loop {
            if self.inner.is_closed() {
                return Err(SendError(item));
            }

            let tail = self.inner.tail.load(Ordering::Relaxed);
            let head = self.inner.head.load(Ordering::Acquire);

            let next = (tail + 1) % self.inner.capacity;
            if next != head {
                // Safety: Producer has exclusive access to this slot
                unsafe {
                    (*self.inner.buffer[tail].get()).write(item);
                }
                self.inner.tail.store(next, Ordering::Release);
                self.inner.waker.wake();
                return Ok(());
            }
            poll_fn(|cx| {
                self.inner.waker.register(cx.waker());

                let tail = self.inner.tail.load(Ordering::Relaxed);
                let head = self.inner.head.load(Ordering::Acquire);
                if (tail + 1) % self.inner.capacity != head {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;
        }
    }

    pub fn close(&self) {
        self.inner.closed.store(true, Ordering::Release);
        self.inner.waker.wake();
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.close()
    }
}
