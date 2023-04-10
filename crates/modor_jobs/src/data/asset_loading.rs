use crate::{platform, Job};
use std::any::Any;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::future::Future;

/// Name of the asset folder taken into account in the folder `CARGO_MANIFEST_DIR`.
pub const ASSET_FOLDER_NAME: &str = "assets";

/// An asynchronous job to retrieve an asset file.
///
/// # Example
///
/// ```
/// # use std::path::*;
/// # use modor::*;
/// # use modor_jobs::*;
/// #
/// #[derive(Component)]
/// struct AssetMetadata {
///     job: AssetLoadingJob<usize>,
///     size: Result<usize, AssetMetadataError>
/// }
///
/// #[systems]
/// impl AssetMetadata {
///     fn new(path: impl AsRef<str>) -> Self {
///         Self {
///             job: AssetLoadingJob::new(path, |b| async move { b.len() }),
///             size: Err(AssetMetadataError::NotReadYet),
///         }
///     }
///
///     fn size(&self) -> Result<usize, AssetMetadataError> {
///         self.size
///     }
///
///     #[run]
///     fn poll(&mut self) {
///         match self.job.try_poll() {
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
#[derive(Debug)]
pub struct AssetLoadingJob<T> {
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
                match platform::load_asset(asset_path).await {
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
