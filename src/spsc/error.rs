use std::{error::Error, fmt::Debug};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("send failed, channel is closed")]
pub struct SendError<T>(pub T);

impl<T> From<SendError<T>> for Box<dyn Error + Send>
where
    T: Send + Debug + 'static,
{
    fn from(err: SendError<T>) -> Self {
        Box::new(err)
    }
}
