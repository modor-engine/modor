use crate::platform;
use crate::platform::{JobFutureJoinHandle, VariableSend};
use futures::channel::oneshot;
use futures::channel::oneshot::{Receiver, Sender};
use std::any;
use std::any::Any;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::future::Future;

/// An asynchronous job.
///
/// # Example
///
/// ```rust
/// # use std::path::{Path, PathBuf};
/// # use modor::*;
/// # use modor_jobs::*;
/// #
/// struct FileReader {
///     job: Job<Vec<u8>>,
///     bytes: Result<Vec<u8>, FileReaderError>,
/// }
///
/// impl FileReader {
///     fn new(path: impl Into<PathBuf>) -> Self {
///         let path = path.into();
///         Self {
///             job: Job::new(async {
///                 async_std::fs::read(path)
///                     .await
///                     .expect("cannot read file")
///             }),
///             bytes: Err(FileReaderError::NotReadYet),
///         }
///     }
///
///     fn bytes(&self) -> Result<&[u8], &FileReaderError> {
///         self.bytes.as_ref().map(Vec::as_slice)
///     }
///
///     fn poll(&mut self) {
///         match self.job.try_poll() {
///             Ok(Some(result)) => self.bytes = Ok(result),
///             Ok(None) => (),
///             Err(_) => self.bytes = Err(FileReaderError::IoError),
///         }
///     }
/// }
///
/// enum FileReaderError {
///     NotReadYet,
///     IoError,
/// }
/// ```
#[derive(Debug)]
pub struct Job<T> {
    receiver: Option<Receiver<T>>,
    _join: JobFutureJoinHandle<()>,
}

impl<T> Job<T>
where
    T: Any + VariableSend + Debug,
{
    /// Creates a new job to run a `future`.
    ///
    /// # Panics
    ///
    /// The future will panic if the [`Job`](Job) is dropped before the future has finished.
    pub fn new(future: impl JobFuture<T>) -> Self {
        let (sender, receiver) = oneshot::channel();
        let job = Self::job_future(future, sender);
        let join = platform::spawn_future(job);
        debug!(
            "job producing value of type `{}` has started", // no-coverage
            any::type_name::<T>()                           // no-coverage
        );
        Self {
            receiver: Some(receiver),
            _join: join,
        }
    }

    #[allow(clippy::future_not_send)]
    async fn job_future(future: impl JobFuture<T>, sender: Sender<T>) {
        sender
            .send(future.await)
            .expect("job dropped before future finishes");
    }

    /// Try polling the job result.
    ///
    /// `None` is returned if the result is not yet available or has already been retrieved.
    ///
    /// # Errors
    ///
    /// An error is returned if the future run by a [`Job`](Job) has panicked.
    pub fn try_poll(&mut self) -> Result<Option<T>, JobPanickedError> {
        if let Some(receiver) = &mut self.receiver {
            let result = receiver.try_recv().map_err(|_| JobPanickedError);
            if let Ok(Some(_)) | Err(_) = &result {
                self.receiver = None;
                debug!(
                    "job producing value of type `{}` has finished", // no-coverage
                    any::type_name::<T>()                            // no-coverage
                );
            } else {
                trace!(
                    "job producing value of type `{}` still in progress", // no-coverage
                    any::type_name::<T>()                                 // no-coverage
                );
            }
            result
        } else {
            debug!(
                "job result of type `{}` already retrieved", // no-coverage
                any::type_name::<T>()                        // no-coverage
            );
            Ok(None)
        }
    }
}

/// An error occurring when the future run by a [`Job`](Job) panics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JobPanickedError;

// coverage: off (not necessary to test Display impl)
impl Display for JobPanickedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "job has panicked")
    }
}
// coverage: on

impl Error for JobPanickedError {}

/// A trait implemented for any future runnable by a job that produces a value of type `T`.
pub trait JobFuture<T>: Future<Output = T> + VariableSend + Any {}

impl<F, T> JobFuture<T> for F where F: Future<Output = T> + VariableSend + Any {}
