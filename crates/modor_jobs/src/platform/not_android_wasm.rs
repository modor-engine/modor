use crate::{AssetLoadingError, ASSET_FOLDER_NAME};
use async_std::task;
use async_std::task::JoinHandle;
use std::any::Any;
use std::env;
use std::future::Future;

pub trait JobFuture<T>: Future<Output = T> + Send + Any {}

impl<F, T> JobFuture<T> for F where F: Future<Output = T> + Send + Any {}

pub(crate) type JobFutureJoinHandle<T> = JoinHandle<T>;

pub(crate) fn spawn_future(future: impl JobFuture<()>) -> JobFutureJoinHandle<()> {
    task::spawn(future)
}

pub(crate) async fn load_asset(path: String) -> Result<Vec<u8>, AssetLoadingError> {
    let base_path = if let Some(path) = env::var_os("CARGO_MANIFEST_DIR") {
        path.into()
    } else {
        env::current_exe()
            .map_err(|e| AssetLoadingError::IoError(e.to_string()))?
            .parent()
            .expect("internal error: cannot retrieve executable folder")
            .to_path_buf()
    };
    async_std::fs::read(base_path.join(ASSET_FOLDER_NAME).join(path))
        .await
        .map_err(|e| AssetLoadingError::IoError(e.to_string()))
}
