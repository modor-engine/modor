use crate::AssetLoadingError;
use async_std::task;
use async_std::task::JoinHandle;
use std::any::Any;
use std::ffi::CString;
use std::future::Future;
use std::io::ErrorKind;

/// A trait implemented for any future runnable by a job that produces a value of type `T`.
pub trait JobFuture<T>: Future<Output = T> + Send + Any {}

impl<F, T> JobFuture<T> for F where F: Future<Output = T> + Send + Any {}

pub(crate) type JobFutureJoinHandle<T> = JoinHandle<T>;

pub(crate) fn spawn_future(future: impl JobFuture<()>) -> JobFutureJoinHandle<()> {
    task::spawn(future)
}

#[allow(clippy::unused_async)]
pub(crate) async fn load_asset(path: String) -> Result<Vec<u8>, AssetLoadingError> {
    let path = CString::new(path.into_bytes()).map_err(|_| AssetLoadingError::InvalidAssetPath)?;
    ndk_glue::native_activity()
        .asset_manager()
        .open(&path)
        .ok_or_else(|| AssetLoadingError::IoError(ErrorKind::NotFound.to_string()))?
        .get_buffer()
        .map_err(|e| AssetLoadingError::IoError(e.to_string()))
        .map(<[u8]>::to_vec)
}
