use std::marker::PhantomData;

pub struct SendError<T> {
    _a: PhantomData<T>,
}
