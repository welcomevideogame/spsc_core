use std::{
    error::Error,
    fmt::{Debug, Display},
};

#[derive(Debug, Clone)]
pub struct SendError<T>(pub T);

impl<T> Display for SendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "send failed, channel is closed")
    }
}

impl<T: Debug> Error for SendError<T> {}
