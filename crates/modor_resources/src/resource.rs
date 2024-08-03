use derivative::Derivative;
use modor::log::error;
use modor::{App, FromApp, Glob, Global};
use modor_jobs::{AssetLoadingError, AssetLoadingJob, Job};
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::{any, fmt};

/// A resource loaded from a path or a custom source.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_resources::*;
/// #
/// // Definition
///
/// #[derive(Default)]
/// struct ContentSize {
///     size: Option<usize>,
/// }
///
/// impl Resource for ContentSize {
///     type Source = ContentSizeSource;
///     type Loaded = ContentSizeLoaded;
///
///     fn load_from_file(file_bytes: Vec<u8>) -> Result<Self::Loaded, ResourceError> {
///         Ok(ContentSizeLoaded {
///             size_in_bytes: file_bytes.len()
///         })
///     }
///
///     fn load(source: &Self::Source) -> Result<Self::Loaded, ResourceError> {
///         Ok(ContentSizeLoaded {
///             size_in_bytes: match source {
///                 ContentSizeSource::Str(str) => str.len(),
///                 ContentSizeSource::String(str) => str.len(),
///             }
///         })
///     }
///
///     fn update(&mut self, app: &mut App, loaded: Option<Self::Loaded>, source: &ResSource<Self>) {
///         if let Some(loaded) = loaded {
///             self.size = Some(loaded.size_in_bytes);
///             println!("Resource has been successfully loaded from `{source:?}`");
///         }
///     }
/// }
///
/// #[non_exhaustive]
/// #[derive(Clone, Debug)]
/// enum ContentSizeSource {
///     Str(&'static str),
///     String(String),
/// }
///
/// impl Source for ContentSizeSource {
///     fn is_async(&self) -> bool {
///         false
///     }
/// }
///
/// struct ContentSizeLoaded {
///     size_in_bytes: usize,
/// }
///
/// // Usage
///
/// struct Content {
///     size: Res<ContentSize>,
/// }
///
/// impl Content {
///     fn new(app: &mut App) -> Self {
///         Self {
///             size: ContentSize::default().load_from_path(app, "path/to/content"),
///         }
///     }
///
///     fn update(&mut self, app: &mut App) {
///         self.size.update(app);
///         if let (Some(size), ResourceState::Loaded) = (self.size.size, self.size.state()) {
///             println!("Content size: {}", size);
///         }
///     }
/// }
/// ```
#[derive(Derivative)]
#[derivative(Debug(bound = "T: Debug, T::Source: Debug"))]
pub struct Res<T: Resource> {
    inner: T,
    source: ResSource<T>,
    loading: Option<Loading<T>>,
    version: u64,
    state: ResourceState,
    properties: Glob<ResProperties>,
}

impl<T> Deref for Res<T>
where
    T: Resource,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Res<T>
where
    T: Resource,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Res<T>
where
    T: Resource,
{
    /// Updates the resource.
    pub fn update(&mut self, app: &mut App) {
        let mut latest_loaded = None;
        match self.loading.take() {
            Some(Loading::Path(mut job)) => match job.try_poll() {
                Ok(Some(Ok(loaded))) => latest_loaded = Some(self.success(loaded)),
                Ok(Some(Err(err))) => self.fail(err),
                Ok(None) => self.loading = Some(Loading::Path(job)),
                Err(err) => self.fail(ResourceError::Loading(err)),
            },
            Some(Loading::Source(mut job)) => match job.try_poll() {
                Ok(Some(Ok(loaded))) => latest_loaded = Some(self.success(loaded)),
                Ok(Some(Err(err))) => self.fail(err),
                Ok(None) => self.loading = Some(Loading::Source(job)),
                Err(err) => self.fail(ResourceError::Other(err.to_string())),
            },
            Some(Loading::Sync(loaded)) => latest_loaded = Some(self.success(loaded)),
            None => (),
        }
        self.inner.update(app, latest_loaded, &self.source);
        self.properties.get_mut(app).state = self.state.clone();
    }

    /// Returns the state of the resource.
    pub fn state(&self) -> &ResourceState {
        &self.state
    }

    /// Starts resource reloading.
    ///
    /// During reloading and in case the reloading fails, the previously loaded resource is
    /// still used.
    pub fn reload(&mut self) {
        self.state = ResourceState::Loading;
        self.loading = None;
        match &self.source {
            ResSource::Path(path) => {
                self.loading = Some(Loading::Path(AssetLoadingJob::new(path, |t| async {
                    T::load_from_file(t)
                })));
            }
            ResSource::Source(source) => {
                if source.is_async() {
                    let source = source.clone();
                    self.loading = Some(Loading::Source(Job::new(async move { T::load(&source) })));
                } else {
                    match T::load(source) {
                        Ok(loaded) => self.loading = Some(Loading::Sync(loaded)),
                        Err(err) => self.fail(err),
                    }
                }
            }
        }
    }

    /// Starts resource reloading from a different `path`.
    ///
    /// During reloading and in case the reloading fails, the previously loaded resource is
    /// still used.
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
    pub fn reload_with_path(&mut self, path: impl Into<String>) {
        self.source = ResSource::Path(path.into());
        self.reload();
    }

    /// Starts resource reloading from a different `source`.
    ///
    /// During reloading and in case the reloading fails, the previously loaded resource is
    /// still used.
    pub fn reload_with_source(&mut self, source: T::Source) {
        self.source = ResSource::Source(source);
        self.reload();
    }

    fn success(&mut self, loaded: T::Loaded) -> T::Loaded {
        self.state = ResourceState::Loaded;
        loaded
    }

    fn fail(&mut self, err: ResourceError) {
        error!(
            "Failed to load resource of type `{}` from `{:?}`: {err}",
            any::type_name::<T>(),
            self.source,
        );
        self.state = ResourceState::Error(err);
    }
}

/// A trait implemented for types that can be used to configure a [`Res`].
pub trait ResLoad: Sized + Resource {
    /// Load a resource from a `path`.
    ///
    /// Resource loading is asynchronous.
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
    fn load_from_path(self, app: &mut App, path: impl Into<String>) -> Res<Self>;

    /// Load a resource from a custom `source`.
    ///
    /// Resource loading is asynchronous if [`Self::Source::is_async()`](Source::is_async())
    /// returns `true`.
    fn load_from_source(self, app: &mut App, source: Self::Source) -> Res<Self>;
}

impl<T> ResLoad for T
where
    T: Resource,
{
    fn load_from_path(self, app: &mut App, path: impl Into<String>) -> Res<Self> {
        let mut res = Res {
            inner: self,
            source: ResSource::Path(path.into()),
            loading: None,
            version: 0,
            state: ResourceState::Loading,
            properties: Glob::from_app(app),
        };
        res.reload();
        res
    }

    fn load_from_source(self, app: &mut App, source: Self::Source) -> Res<Self> {
        let mut res = Res {
            inner: self,
            source: ResSource::Source(source),
            loading: None,
            version: 0,
            state: ResourceState::Loading,
            properties: Glob::from_app(app),
        };
        res.reload();
        res
    }
}

/// The state of a [`Res`].
///
/// # Examples
///
/// See [`Res`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ResourceState {
    /// The resource is loading.
    #[default]
    Loading,
    /// The resource is loaded.
    Loaded,
    /// The resource loading has failed.
    Error(ResourceError),
}

impl ResourceState {
    /// Returns the error in case of failed loading.
    pub fn error(&self) -> Option<&ResourceError> {
        match self {
            Self::Loading | Self::Loaded => None,
            Self::Error(err) => Some(err),
        }
    }
}

/// A trait for defining a resource.
///
/// # Examples
///
/// See [`Res`].
pub trait Resource: Sized {
    /// The custom source type.
    type Source: Source;
    /// The loaded resource type.
    type Loaded: Send + 'static;

    /// Loads the resource from file bytes.
    ///
    /// This method is called when the resource is loaded using a path.
    ///
    /// # Errors
    ///
    /// An error is returned if the resource cannot be loaded.
    fn load_from_file(file_bytes: Vec<u8>) -> Result<Self::Loaded, ResourceError>;

    /// Loads the resource from a custom `source`.
    ///
    /// # Errors
    ///
    /// An error is returned if the resource cannot be loaded.
    fn load(source: &Self::Source) -> Result<Self::Loaded, ResourceError>;

    /// Updates the resource.
    ///
    /// In case resource loaded has just finished, `loaded` is `Some`.
    ///
    /// `label` can be used for logging.
    fn update(&mut self, app: &mut App, loaded: Option<Self::Loaded>, source: &ResSource<Self>);
}

/// A trait for defining a source used to load a [`Resource`].
pub trait Source: Clone + Send + Debug + 'static {
    /// Returns whether the resource is loaded asynchronously.
    fn is_async(&self) -> bool;
}

/// An error that occurs during resource loading.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResourceError {
    /// There was an error while loading the asset.
    Loading(AssetLoadingError),
    /// There was an error while loading or parsing the resource.
    Other(String),
}

impl Display for ResourceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Loading(e) => write!(f, "asset loading error: {e}"),
            Self::Other(e) => write!(f, "resource loading error: {e}"),
        }
    }
}

/// The properties of a [`Res`].
#[derive(Debug, FromApp, Global)]
#[non_exhaustive]
pub struct ResProperties {
    /// State of the resource.
    pub state: ResourceState,
}

/// The source of a [`Res`].
#[derive(Derivative)]
#[derivative(Debug(bound = ""))]
pub enum ResSource<T: Resource> {
    /// A path.
    Path(String),
    /// A custom source.
    Source(T::Source),
}

#[derive(Derivative)]
#[derivative(Debug)]
enum Loading<T: Resource> {
    Path(#[derivative(Debug = "ignore")] AssetLoadingJob<Result<T::Loaded, ResourceError>>),
    Source(#[derivative(Debug = "ignore")] Job<Result<T::Loaded, ResourceError>>),
    Sync(#[derivative(Debug = "ignore")] T::Loaded),
}
