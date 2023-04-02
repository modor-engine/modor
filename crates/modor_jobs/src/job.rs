use futures::channel::oneshot;
use futures::channel::oneshot::Receiver;
use std::any;
use std::any::Any;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::future::Future;

macro_rules! job_future {
    ($future:ident, $sender:ident) => {
        async {
            $sender.send($future.await).is_err().then(|| {
                panic!(
                    "job producing value of type {} dropped before future finishes",
                    any::type_name::<T>()
                )
            });
        }
    };
}

/// An asynchronous job.
///
/// # Example
///
/// ```
/// # use std::path::{Path, PathBuf};
/// # use modor::*;
/// # use modor_jobs::*;
/// #
/// #[derive(Component)]
/// struct FileReader {
///     job: Job<Vec<u8>>,
///     bytes: Result<Vec<u8>, FileReaderError>
/// }
///
/// #[systems]
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
///     #[run]
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
///     IoError
/// }
/// ```
#[derive(Debug)]
pub struct Job<T> {
    receiver: Option<Receiver<T>>,
    #[cfg(not(target_arch = "wasm32"))]
    _join: async_std::task::JoinHandle<()>,
}

impl<T> Job<T>
where
    T: Any + Send + Debug,
{
    /// Creates a new job to run a `future`.
    ///
    /// # Panics
    ///
    /// The future will panic the [`Job`](crate::Job) is dropped before the future has finished.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = T> + Send + Any,
    {
        let (sender, receiver) = oneshot::channel();
        let job = job_future!(future, sender);
        let join = async_std::task::spawn(job);
        debug!(
            "job producing value of type `{}` has started",
            any::type_name::<T>()
        );
        Self {
            receiver: Some(receiver),
            _join: join,
        }
    }

    /// Creates a new job to run a `future`.
    ///
    /// # Panics
    ///
    /// The future will panic the [`Job`](crate::Job) is dropped before the future has finished.
    #[cfg(target_arch = "wasm32")]
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = T> + Any,
    {
        let (sender, receiver) = oneshot::channel();
        let job = job_future!(future, sender);
        wasm_bindgen_futures::spawn_local(job);
        debug!(
            "job producing value of type `{}` has started",
            any::type_name::<T>()
        );
        Self {
            receiver: Some(receiver),
        }
    }

    /// Try polling the job result.
    ///
    /// `None` is returned if the result is not yet available or has already been retrieved.
    ///
    /// # Errors
    ///
    /// An error is returned if the future run by a [`Job`](crate::Job) has panicked.
    pub fn try_poll(&mut self) -> Result<Option<T>, JobPanickedError> {
        if let Some(receiver) = &mut self.receiver {
            let result = receiver.try_recv().map_err(|_| JobPanickedError);
            if let Ok(Some(_)) | Err(_) = &result {
                self.receiver = None;
                debug!(
                    "job producing value of type `{}` has finished",
                    any::type_name::<T>()
                );
            } else {
                trace!(
                    "job producing value of type `{}` still in progress",
                    any::type_name::<T>()
                );
            }
            result
        } else {
            debug!(
                "job result of type `{}` already retrieved",
                any::type_name::<T>()
            );
            Ok(None)
        }
    }
}

/// An error occurring when the future run by a [`Job`](crate::Job) panics.
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
