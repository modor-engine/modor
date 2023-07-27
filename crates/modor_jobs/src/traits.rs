use modor::VariableSend;
use std::any::Any;
use std::future::Future;

/// A trait implemented for any future runnable by a job that produces a value of type `T`.
pub trait JobFuture<T>: Future<Output = T> + VariableSend + Any {}

impl<F, T> JobFuture<T> for F where F: Future<Output = T> + VariableSend + Any {}
