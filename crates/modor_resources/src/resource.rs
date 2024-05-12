use derivative::Derivative;
use modor::log::error;
use modor::{Context, Glob, GlobRef, Node, Visit};
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
///     unit: ContentSizeUnit,
/// }
///
/// impl Resource for ContentSize {
///     type Source = ContentSizeSource;
///     type Loaded = ContentSizeLoaded;
///     type Glob = ContentSizeGlob;
///
///     fn should_be_reloaded(
///         &self,
///         glob: &Glob<Option<Self::Glob>>,
///         ctx: &mut Context<'_>
///     ) -> bool {
///         false
///     }
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
///     fn update(
///         &mut self,
///         glob: &Glob<Option<Self::Glob>>,
///         ctx: &mut Context<'_>,
///         loaded: Option<Self::Loaded>
///     ) {
///         if let Some(loaded) = loaded {
///             *glob.get_mut(ctx) = Some(ContentSizeGlob {
///                 size: match self.unit {
///                     ContentSizeUnit::Bytes => loaded.size_in_bytes,
///                     ContentSizeUnit::Bits => loaded.size_in_bytes * 8,
///                 },
///                 unit: self.unit,
///             });
///         }
///     }
/// }
///
/// #[derive(Default, Debug, Clone, Copy)]
/// enum ContentSizeUnit {
///     #[default]
///     Bytes,
///     Bits,
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
/// struct ContentSizeGlob {
///     size: usize,
///     unit: ContentSizeUnit,
/// }
///
/// // Usage
///
/// #[derive(Visit)]
/// struct Content {
///     size: Res<ContentSize>,
/// }
///
/// impl Content {
///     fn new(ctx: &mut Context) -> Self {
///         Self {
///             size: Res::from_path(ctx, "path/to/content"),
///         }
///     }
/// }
///
/// impl Node for Content {
///     fn on_enter(&mut self, ctx: &mut Context<'_>) {
///         if let Some(size) = self.size.glob().get(ctx) {
///             println!("Content size: {} {:?}", size.size, size.unit);
///         }
///     }
/// }
/// ```
#[derive(Debug, Visit)]
pub struct Res<T: Resource> {
    inner: T,
    glob: Glob<Option<T::Glob>>,
    location: ResourceLocation<T>,
    loading: Option<Loading<T>>,
    err: Option<ResourceError>,
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

impl<T> Node for Res<T>
where
    T: Resource,
{
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        if self.inner.should_be_reloaded(&self.glob, ctx) {
            self.reload();
        }
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
        self.inner.update(&self.glob, ctx, latest_loaded);
    }
}

impl<T> Res<T>
where
    T: Resource,
{
    /// Creates a resource from a `path`.
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
    pub fn from_path(ctx: &mut Context<'_>, path: impl Into<String>) -> Self {
        let mut res = Self {
            inner: T::default(),
            glob: Glob::new(ctx, None),
            location: ResourceLocation::Path(path.into()),
            loading: None,
            err: None,
        };
        res.reload();
        res
    }

    /// Creates a resource from a custom `source`.
    ///
    /// Resource loading is asynchronous if [`T::Source::is_async()`](Source::is_async())
    /// returns `true`.
    pub fn from_source(ctx: &mut Context<'_>, source: T::Source) -> Self {
        let mut res = Self {
            inner: T::default(),
            glob: Glob::new(ctx, None),
            location: ResourceLocation::Source(source),
            loading: None,
            err: None,
        };
        res.reload();
        res
    }

    /// Returns a reference to global data.
    pub fn glob(&self) -> &GlobRef<Option<T::Glob>> {
        self.glob.as_ref()
    }

    /// Returns the error in case the loading has failed.
    pub fn err(&self) -> Option<&ResourceError> {
        self.err.as_ref()
    }

    /// Starts resource reloading.
    ///
    /// During reloading and in case the reloading fails, the previously loaded resource is
    /// still used.
    pub fn reload(&mut self) {
        match &self.location {
            ResourceLocation::Path(path) => {
                self.loading = Some(Loading::Path(AssetLoadingJob::new(path, |t| async {
                    T::load_from_file(t)
                })));
            }
            ResourceLocation::Source(source) => {
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
        self.location = ResourceLocation::Path(path.into());
        self.reload();
    }

    /// Starts resource reloading from a different `source`.
    ///
    /// During reloading and in case the reloading fails, the previously loaded resource is
    /// still used.
    pub fn reload_with_source(&mut self, source: T::Source) {
        self.location = ResourceLocation::Source(source);
        self.reload();
    }

    fn success(&mut self, loaded: T::Loaded) -> T::Loaded {
        self.err = None;
        loaded
    }

    fn fail(&mut self, err: ResourceError) {
        match &self.location {
            ResourceLocation::Path(path) => error!(
                "failed to load `{}` resource with path `{}`: {err}",
                any::type_name::<T>(),
                path
            ),
            ResourceLocation::Source(source) => error!(
                "failed to load `{}` resource with source `{:?}`: {err}",
                any::type_name::<T>(),
                source
            ),
        }
        self.err = Some(err);
    }
}

/// A trait for defining a resource.
///
/// # Examples
///
/// See [`Res`].
pub trait Resource: Default {
    /// The custom source type.
    type Source: Source;
    /// The loaded resource type.
    type Loaded: Send + 'static;
    /// The global data of the resource.
    type Glob: 'static;

    /// Returns whether the resource should be reloaded.
    fn should_be_reloaded(&self, glob: &Glob<Option<Self::Glob>>, ctx: &mut Context<'_>) -> bool;

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

    /// Updates the resource during node update.
    ///
    /// In case resource loaded has just finished, `loaded` is `Some`.
    fn update(
        &mut self,
        glob: &Glob<Option<Self::Glob>>,
        ctx: &mut Context<'_>,
        loaded: Option<Self::Loaded>,
    );
}

/// A trait for defining a source used to load a [`Resource`].
pub trait Source: Debug + Clone + Send + 'static {
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

#[derive(Debug)]
enum ResourceLocation<T: Resource> {
    Path(String),
    Source(T::Source),
}

#[derive(Visit, Derivative)]
#[derivative(Debug)]
enum Loading<T: Resource> {
    Path(#[derivative(Debug = "ignore")] AssetLoadingJob<Result<T::Loaded, ResourceError>>),
    Source(#[derivative(Debug = "ignore")] Job<Result<T::Loaded, ResourceError>>),
    Sync(#[derivative(Debug = "ignore")] T::Loaded),
}
