use std::{
    future::poll_fn,
    pin::Pin,
    sync::{Arc, atomic::Ordering},
    task::Poll,
};

use futures::Stream;

use crate::spsc::Inner;

/// The receiving half of a single-producer, single-consumer asynchronous channel.
///
/// `Receiver<T>` allows the consumer task to asynchronously receive values from the channel.
///
/// # Behavior
/// - Only one `Receiver` should exist per channel; cloning is **not allowed**.
/// - If the buffer is empty, `recv` will asynchronously wait until an item is sent.
/// - Once the channel is closed and all items are consumed, `recv` will return `None`.
///
/// # Example
/// ```
/// use spsc::channel;
///
/// let (tx, rx) = channel::<i32>(16);
///
/// // In the consumer task
/// tokio::spawn(async move {
///     if let Some(value) = rx.recv().await {
///         println!("Received: {}", value);
///     }
/// });
/// ```
#[derive(Debug)]
pub struct Receiver<T> {
    pub(super) inner: Arc<Inner<T>>,
}

impl<T> Receiver<T> {
    /// Receives a value from the channel.
    ///
    /// Returns `Some(value)` if value is available, or `None` if the channel
    /// has been closed and all values have been consumed. If empty, it will
    /// asynchronously wait until a value is received.

    pub async fn recv(&self) -> Option<T> {
        loop {
            let head = self.inner.head.load(Ordering::Relaxed);
            let tail = self.inner.tail.load(Ordering::Acquire);

            if head == tail {
                if self.inner.closed.load(Ordering::Acquire) {
                    return None;
                }

                poll_fn(|cx| {
                    self.inner.waker.register(cx.waker());

                    let head = self.inner.head.load(Ordering::Relaxed);
                    let tail = self.inner.tail.load(Ordering::Acquire);

                    if head != tail {
                        Poll::Ready(())
                    } else if self.inner.closed.load(Ordering::Acquire) {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                })
                .await;

                let head = self.inner.head.load(Ordering::Relaxed);
                let tail = self.inner.tail.load(Ordering::Acquire);
                if head == tail && self.inner.closed.load(Ordering::Acquire) {
                    return None;
                }

                continue;
            }

            // Safety: Consumer has exclusive access to this slot
            let value = unsafe { (*self.inner.buffer[head].get()).assume_init_read() };

            self.inner
                .head
                .store((head + 1) % self.inner.capacity, Ordering::Release);

            self.inner.waker.wake();
            return Some(value);
        }
    }
}

/// An asynchronous `Stream` wrapper around a `Receiver`.
///
/// `ReceiverStream` allows a `Receiver<T>` from a single-producer, single-consumer
/// channel to be used as a standard `futures::Stream`. Each call to `poll_next`
/// will attempt to receive the next item from the underlying `Receiver`.
///
/// # Behavior
/// - The stream yields items in the same order they are sent into the channel.
/// - If the channel is empty, the stream will yield `Poll::Pending` until an item
///   is available.
/// - Once the channel is closed and all items have been consumed, the stream
///   returns `Poll::Ready(None)` permanently.
///
/// # Lifetime
/// The stream holds a reference to the original `Receiver`. The `'a` lifetime
/// ensures the `Receiver` lives at least as long as the stream.
///
/// # Example
/// ```
/// use spsc::channel;
/// use futures::stream::StreamExt;
/// use tokio::spawn;
///
/// let (tx, rx) = channel::<i32>(16);
/// let mut stream = ReceiverStream::new(&rx);
///
/// tokio::spawn(async move { tx.send(42).await.unwrap(); });
///
/// if let Some(value) = stream.next().await {
///     assert_eq!(value, 42);
/// }
/// ```
pub struct ReceiverStream<'a, T> {
    /// Reference to the underlying `Receiver`.
    receiver: &'a Receiver<T>,

    /// Stores a pending `Future` for the next value to be polled.
    ///
    /// This is used to implement the `Stream` interface by repeatedly
    /// polling the receiver's async `recv` method.
    pending: Option<Pin<Box<dyn Future<Output = Option<T>> + Send + 'a>>>,
}

impl<'a, T> ReceiverStream<'a, T> {
    /// Creates a new `ReceiverStream` wrapping the given `Receiver`.
    ///
    /// # Arguments
    /// - `rx`: A reference to the channel's receiver. The reference must
    ///   outlive the `ReceiverStream`.
    pub fn new(rx: &'a Receiver<T>) -> Self {
        Self {
            receiver: rx,
            pending: None,
        }
    }
}

impl<'a, T> Stream for ReceiverStream<'a, T>
where
    T: Send + Sync + 'a,
{
    type Item = T;

    /// Attempts to pull the next value from the underlying receiver.
    ///
    /// If there is a value available, returns `Poll::Ready(Some(value))`.
    /// If the receiver is empty, returns `Poll::Pending`. Once the channel
    /// is closed and empty, returns `Poll::Ready(None)` permanently.
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if self.pending.is_none() {
            let fut = self.receiver.recv();
            self.pending = Some(Box::pin(fut));
        }

        let fut = self.pending.as_mut().unwrap();
        match fut.as_mut().poll(cx) {
            Poll::Ready(opt) => {
                self.pending = None;
                Poll::Ready(opt)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.inner.closed.store(true, Ordering::Release);
        self.inner.waker.wake();
    }
}
