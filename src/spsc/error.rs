use std::error::Error;
use std::fmt::Debug;
use thiserror::Error;

/// Error returned when sending a value into a channel fails because the channel is closed.
///
/// `SendError<T>` wraps the value that was being sent, allowing the caller
/// to recover or retry the item if desired.
///
/// # Example
/// ```
/// use spsc::channel;
/// use spsc::SendError;
///
/// let (tx, rx) = channel::<i32>(2);
///
/// // Close the channel manually (or drop receiver)
/// rx.close();
///
/// let result: Result<(), i32> = tx.send(42).await;
/// if let Err(SendError(value)) = result {
///     println!("Failed to send value: {}", value);
/// }
/// ```
#[derive(Debug, Clone, Error, PartialEq)]
#[error("send failed, channel is closed")]
pub struct SendError<T>(
    /// The value that failed to be sent
    pub T,
);

impl<T> From<SendError<T>> for Box<dyn Error + Send>
where
    T: Send + Debug + 'static,
{
    /// Converts a `SendError<T>` into a boxed dynamic error (`Box<dyn Error + Send>`).
    ///
    /// This is useful when you want to treat channel send failures
    /// as a generic error type that can be propagated or stored.
    ///
    /// # Example
    /// ```
    /// use spsc::SendError;
    /// use std::error::Error;
    ///
    /// let err: SendError<i32> = SendError(42);
    /// let boxed: Box<dyn Error + Send> = err.into();
    /// ```
    fn from(err: SendError<T>) -> Self {
        Box::new(err)
    }
}
