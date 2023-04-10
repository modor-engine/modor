use crate::{AssetLoadingError, ASSET_FOLDER_NAME};
use std::any::Any;
use std::future::Future;
use std::marker::PhantomData;

pub trait JobFuture<T>: Future<Output = T> + Any {}

impl<F, T> JobFuture<T> for F where F: Future<Output = T> + Any {}

pub(crate) type JobFutureJoinHandle<T> = PhantomData<T>;

pub(crate) fn spawn_future(future: impl JobFuture<()>) -> JobFutureJoinHandle<()> {
    wasm_bindgen_futures::spawn_local(future);
    PhantomData
}

#[allow(clippy::future_not_send)]
pub(crate) async fn load_asset(path: String) -> Result<Vec<u8>, AssetLoadingError> {
    let base_url = web_sys::window()
        .ok_or(AssetLoadingError::NotFoundDomWindow)?
        .location()
        .href()
        .map_err(|e| AssetLoadingError::InvalidLocationHref(format!("{e:?}")))?;
    let url = format!("{base_url}/{ASSET_FOLDER_NAME}/{path}");
    reqwest::get(url)
        .await
        .map_err(|e| AssetLoadingError::IoError(e.to_string()))?
        .error_for_status()
        .map_err(|e| AssetLoadingError::IoError(e.to_string()))?
        .bytes()
        .await
        .map_err(|e| AssetLoadingError::IoError(e.to_string()))
        .map(Into::into)
}
