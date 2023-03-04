use crate::Job;
use std::any::Any;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::future::Future;

/// Name of the asset folder taken into account in the folder `CARGO_MANIFEST_DIR`.
pub const ASSET_FOLDER_NAME: &str = "assets";

/// An asynchronous job to retrieve an asset file.
///
/// # Modor
///
/// - **Type**: component
///
/// # Example
///
/// ```
/// # use std::path::{Path, PathBuf};
/// # use modor::{entity, Built, EntityBuilder};
/// # use modor_jobs::AssetLoadingJob;
/// #
/// struct AssetMetadata {
///     size: Result<usize, AssetMetadataError>
/// }
///
/// #[entity]
/// impl AssetMetadata {
///     fn build(path: impl AsRef<str>) -> impl Built<Self> {
///         EntityBuilder::new(Self {
///             size: Err(AssetMetadataError::NotReadYet),
///         })
///         .with(AssetLoadingJob::new(path, |b| async move { b.len() }))
///     }
///
///     fn size(&self) -> Result<usize, AssetMetadataError> {
///         self.size
///     }
///
///     #[run]
///     fn poll(&mut self, job: &mut AssetLoadingJob<usize>) {
///         match job.try_poll() {
///             Ok(Some(result)) => self.size = Ok(result),
///             Ok(None) => (),
///             Err(_) => self.size = Err(AssetMetadataError::LoadingError),
///         }
///     }
/// }
///
/// #[derive(Clone, Copy)]
/// enum AssetMetadataError {
///     NotReadYet,
///     LoadingError
/// }
/// ```
#[derive(Debug, Component)]
pub struct AssetLoadingJob<T>
where
    T: Any + Send + Debug,
{
    /// Actual job instance that can be used to retrieve the job result.
    inner: Job<Result<T, AssetLoadingError>>,
}

impl<T> AssetLoadingJob<T>
where
    T: Any + Send + Debug,
{
    /// Creates a new job to retrieve asset located at `path`, and apply `f` on the bytes of the
    /// file.
    ///
    /// # Platform-specific
    ///
    /// - Web: HTTP GET call is performed to retrieve the file from URL
    /// `{current_browser_url}/assets/{path}`.
    /// - Android: the file is retrieved using the Android
    /// [`AssetManager`](https://developer.android.com/reference/android/content/res/AssetManager).
    /// - Other: if `CARGO_MANIFEST_DIR` environment variable is set (this is the case if the
    /// application is run using a `cargo` command), then the file is retrieved from path
    /// `{CARGO_MANIFEST_DIR}/assets/{path}`. Else, the file path is
    /// `{executable_folder_path}/assets/{path}`.
    pub fn new<F>(path: impl AsRef<str>, f: impl FnOnce(Vec<u8>) -> F + Send + Any) -> Self
    where
        F: Future<Output = T> + Send,
    {
        let asset_path = path.as_ref().to_string();
        Self {
            inner: Job::<Result<T, AssetLoadingError>>::new(async move {
                match Self::load_asset(asset_path).await {
                    Ok(b) => Ok(f(b).await),
                    Err(e) => Err(e),
                }
            }),
        }
    }

    /// Try polling the job result.
    ///
    /// `None` is returned if the result is not yet available or has already been retrieved.
    ///
    /// # Errors
    ///
    /// An error is returned if the asset has not been successfully loaded.
    pub fn try_poll(&mut self) -> Result<Option<T>, AssetLoadingError> {
        self.inner
            .try_poll()
            .expect("internal error: asset loading job has failed")
            .map_or(Ok(None), |result| result.map(|r| Some(r)))
    }

    #[cfg_attr(target_os = "android", allow(clippy::unused_async))]
    #[cfg_attr(target_arch = "wasm32", allow(clippy::future_not_send))]
    async fn load_asset(path: String) -> Result<Vec<u8>, AssetLoadingError> {
        #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
        {
            let base_path = if let Some(path) = std::env::var_os("CARGO_MANIFEST_DIR") {
                path.into()
            } else {
                std::env::current_exe()
                    .map_err(|e| AssetLoadingError::IoError(e.to_string()))?
                    .parent()
                    .expect("internal error: cannot retrieve executable folder")
                    .to_path_buf()
            };
            async_std::fs::read(base_path.join(ASSET_FOLDER_NAME).join(path))
                .await
                .map_err(|e| AssetLoadingError::IoError(e.to_string()))
        }
        #[cfg(target_os = "android")]
        {
            let path = std::ffi::CString::new(path.into_bytes())
                .map_err(|_| AssetLoadingError::InvalidAssetPath)?;
            ndk_glue::native_activity()
                .asset_manager()
                .open(&path)
                .ok_or_else(|| {
                    AssetLoadingError::IoError(std::io::ErrorKind::NotFound.to_string())
                })?
                .get_buffer()
                .map_err(|e| AssetLoadingError::IoError(e.to_string()))
                .map(<[u8]>::to_vec)
        }
        #[cfg(target_arch = "wasm32")]
        {
            let base_url = web_sys::window()
                .ok_or(AssetLoadingError::NotFoundDomWindow)?
                .location()
                .href()
                .map_err(|e| AssetLoadingError::InvalidLocationHref(format!("{e:?}")))?;
            let url = format!("{}/{ASSET_FOLDER_NAME}/{}", base_url, path);
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
    }
}

/// An error occurring during an asset loading job.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AssetLoadingError {
    /// The provided asset path contains unsupported characters.
    InvalidAssetPath,
    /// DOM `Window` object has not been found, can only occurs for web platform.
    NotFoundDomWindow,
    /// `location.href` property cannot be retrieved, can only occurs for web platform.
    InvalidLocationHref(String),
    /// I/O error occurs while retrieving the resource.
    IoError(String),
}

// coverage: off (not necessary to test Display impl)
impl Display for AssetLoadingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAssetPath => write!(f, "invalid asset path"),
            Self::NotFoundDomWindow => write!(f, "DOM window not found"),
            Self::InvalidLocationHref(m) => write!(f, "invalid location.ref property: {m}"),
            Self::IoError(m) => write!(f, "IO error: {m}"),
        }
    }
}
// coverage: on

impl Error for AssetLoadingError {}
