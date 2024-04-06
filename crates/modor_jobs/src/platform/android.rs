use crate::{AssetLoadingError, JobFuture};
use async_std::task;
use async_std::task::JoinHandle;
use std::ffi::CString;
use std::io::ErrorKind;

/// A trait implemented for any type implementing [`Send`], or implemented for any type on Web
/// platform.
pub trait VariableSend: Send {}

impl<T> VariableSend for T where T: Send {}

pub(crate) type JobFutureJoinHandle<T> = JoinHandle<T>;

pub(crate) fn spawn_future(future: impl JobFuture<()>) -> JobFutureJoinHandle<()> {
    task::spawn(future)
}

#[allow(clippy::unused_async)]
pub(crate) async fn load_asset(path: String) -> Result<Vec<u8>, AssetLoadingError> {
    let path = CString::new(path.into_bytes()).map_err(|_| AssetLoadingError::InvalidAssetPath)?;
    modor::ANDROID_APP
        .get()
        .ok_or(AssetLoadingError::InvalidAppInit)?
        .asset_manager()
        .open(&path)
        .ok_or_else(|| AssetLoadingError::IoError(ErrorKind::NotFound.to_string()))?
        .buffer()
        .map_err(|e| AssetLoadingError::IoError(e.to_string()))
        .map(<[u8]>::to_vec)
}
