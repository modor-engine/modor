use crate::Object;
use std::fmt::Debug;
use std::sync::Arc;
use std::{error, result};
use thiserror::Error;

/// The main result type of `modor`.
pub type Result<T> = result::Result<T, Error>;

/// The main error type of `modor`.
#[derive(Clone, Debug, Error)]
pub enum Error {
    /// The requested object has not been found.
    #[error("object with type `{0}` not found by ID")]
    ObjectNotFound(&'static str),
    /// The requested singleton object has not been found.
    #[error("singleton `{0}` not found")]
    SingletonObjectNotFound(&'static str),
    /// The requested object type is already locked, for example by
    /// [`Context::lock_objects`](crate::Context::lock_objects).
    #[error("objects with type `{0}` accessed but already locked")]
    ObjectTypeAlreadyLocked(&'static str),
    /// Any other error.
    #[error("error occurred: {0}")]
    Other(#[from] Arc<dyn error::Error + Sync + Send>),
}

#[derive(Clone, Debug, Error)]
pub(crate) enum InternalError {
    #[error("creation of object with type `{0}` failed")]
    ObjectCreationFailed(&'static str),
}

/// A trait implemented for types that can be converted to a [`Result<T>`].
pub trait IntoResult<T> {
    /// Converts `self` to a result.
    #[allow(clippy::missing_errors_doc)]
    fn into_result(self) -> Result<T>;
}

impl<T> IntoResult<T> for T {
    fn into_result(self) -> Result<T> {
        Ok(self)
    }
}

impl<T> IntoResult<T> for Result<T>
where
    T: Object,
{
    fn into_result(self) -> Self {
        self
    }
}
