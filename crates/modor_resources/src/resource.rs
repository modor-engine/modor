use crate::testing::ResourceStates;
use derivative::Derivative;
use modor::log::error;
use modor::{App, FromApp, Glob, Global, Globals, State};
use modor_jobs::{AssetLoadingError, AssetLoadingJob, Job};
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::{any, fmt};

/// A resource loaded from a path or a custom source.
///
/// # Examples
///
/// ```rust
/// # use std::marker::PhantomData;
/// # use modor::*;
/// # use modor_resources::*;
/// #
/// // Definition
///
/// #[derive(Default, Updater)]
/// struct ContentSize {
///     #[updater(field, for_field)]
///     size: Option<usize>,
///     #[updater(inner_type, field)]
///     res: PhantomData<ResUpdater<ContentSize>>,
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
///     fn load_from_source(source: &Self::Source) -> Result<Self::Loaded, ResourceError> {
///         Ok(ContentSizeLoaded {
///             size_in_bytes: match source {
///                 ContentSizeSource::Str(str) => str.len(),
///                 ContentSizeSource::String(str) => str.len(),
///             }
///         })
///     }
///
///     fn on_load(&mut self, app: &mut App, loaded: Self::Loaded, source: &ResSource<Self>) {
///         self.size = Some(loaded.size_in_bytes);
///         println!("Resource has been successfully loaded from `{source:?}`");
///     }
/// }
///
/// impl ContentSizeUpdater<'_> {
///     fn apply(mut self, app: &mut App, glob: &Glob<Res<ContentSize>>) {
///         self.size.apply(&mut glob.get_mut(app).size);
///         if let Some(res) = self.res.take_value(|| unreachable!()) {
///             res.apply(app, glob);
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
/// #[derive(FromApp)]
/// struct Content {
///     size: Glob<Res<ContentSize>>,
/// }
///
/// impl State for Content {
///     fn init(&mut self, app: &mut App) {
///         ContentSizeUpdater::default()
///             .res(ResUpdater::default().path("path/to/content"))
///             .apply(app, &self.size);
///     }
///
///     fn update(&mut self, app: &mut App) {
///         let size = self.size.get(app);
///         if let (Some(size), ResourceState::Loaded) = (size.size, size.state()) {
///             println!("Content size: {}", size);
///         }
///     }
/// }
/// ```
#[derive(FromApp, Derivative)]
#[derivative(Debug(bound = "T: Debug, T::Source: Debug"))]
pub struct Res<T: Resource> {
    inner: T,
    source: Option<ResSource<T>>,
    loading: Option<Loading<T>>,
    state: ResourceState,
}

impl<T> Global for Res<T>
where
    T: Resource,
{
    fn init(&mut self, app: &mut App, _index: usize) {
        app.create::<ResManager<T>>();
        app.get_mut::<ResourceStates>()
            .are_all_loaded_fns
            .insert(Self::are_all_loaded);
    }
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
    /// Returns the state of the resource.
    pub fn state(&self) -> &ResourceState {
        &self.state
    }

    fn reload(&mut self, app: &mut App) {
        self.state = ResourceState::Loading;
        self.loading = None;
        match &self.source {
            Some(ResSource::Path(path)) => {
                self.loading = Some(Loading::Path(AssetLoadingJob::new(path, |t| async {
                    T::load_from_file(t)
                })));
            }
            Some(ResSource::Source(source)) => {
                if source.is_async() {
                    let source = source.clone();
                    self.loading = Some(Loading::Source(Job::new(async move {
                        T::load_from_source(&source)
                    })));
                } else {
                    match T::load_from_source(source) {
                        Ok(loaded) => self.loading = Some(Loading::Sync(loaded)),
                        Err(err) => self.fail(err),
                    }
                }
            }
            None => self.state = ResourceState::Loaded,
        }
        self.update(app);
    }

    fn update(&mut self, app: &mut App) {
        match self.loading.take() {
            Some(Loading::Path(mut job)) => match job.try_poll() {
                Ok(Some(Ok(loaded))) => self.success(app, loaded),
                Ok(Some(Err(err))) => self.fail(err),
                Ok(None) => self.loading = Some(Loading::Path(job)),
                Err(err) => self.fail(ResourceError::Loading(err)),
            },
            Some(Loading::Source(mut job)) => match job.try_poll() {
                Ok(Some(Ok(loaded))) => self.success(app, loaded),
                Ok(Some(Err(err))) => self.fail(err),
                Ok(None) => self.loading = Some(Loading::Source(job)),
                Err(err) => self.fail(ResourceError::Other(err.to_string())),
            },
            Some(Loading::Sync(loaded)) => self.success(app, loaded),
            None => (),
        }
    }

    fn success(&mut self, app: &mut App, loaded: T::Loaded) {
        let source = self
            .source
            .as_ref()
            .expect("internal error: missing source");
        self.state = ResourceState::Loaded;
        self.inner.on_load(app, loaded, source);
    }

    fn fail(&mut self, err: ResourceError) {
        error!(
            "Failed to load resource of type `{}` from `{:?}`: {err}",
            any::type_name::<T>(),
            self.source,
        );
        self.state = ResourceState::Error(err);
    }

    fn are_all_loaded(app: &mut App) -> bool {
        app.get_mut::<Globals<Self>>()
            .iter()
            .all(|res| res.state() != &ResourceState::Loading)
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
    Loading,
    /// The resource is loaded.
    #[default]
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
pub trait Resource: FromApp + Sized {
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
    fn load_from_source(source: &Self::Source) -> Result<Self::Loaded, ResourceError>;

    /// Updates the resource when loading has successfully finished.
    fn on_load(&mut self, app: &mut App, loaded: Self::Loaded, source: &ResSource<Self>);
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

/// The source of a [`Res`].
#[derive(Derivative)]
#[derivative(Debug(bound = ""), Clone(bound = ""))]
pub enum ResSource<T: Resource> {
    /// A path.
    Path(String),
    /// A custom source.
    Source(T::Source),
}

impl<T> FromApp for ResSource<T>
where
    T: Resource,
    T::Source: FromApp,
{
    fn from_app(app: &mut App) -> Self {
        Self::Source(T::Source::from_app(app))
    }
}

/// An updater for [`Res`].
///
/// # Examples
///
/// See [`Res`].
#[must_use]
#[derive(Derivative)]
#[derivative(Debug(bound = ""), Default(bound = ""))]
pub struct ResUpdater<T: Resource> {
    source: Option<ResSource<T>>,
    reload: bool,
}

impl<T> ResUpdater<T>
where
    T: Resource,
{
    /// Loads the resource from a given `path`.
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
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.source = Some(ResSource::Path(path.into()));
        self
    }

    /// Loads the resource from a given `source`.
    ///
    /// During reloading and in case the reloading fails, the previously loaded resource is
    /// still used.
    pub fn source(mut self, source: impl Into<T::Source>) -> Self {
        self.source = Some(ResSource::Source(source.into()));
        self
    }

    /// Force resource reloading.
    ///
    /// During reloading and in case the reloading fails, the previously loaded resource is
    /// still used.
    pub fn reload(mut self) -> Self {
        self.reload = true;
        self
    }

    /// Runs the update.
    pub fn apply(self, app: &mut App, glob: &Glob<Res<T>>) {
        glob.take(app, |res, app| {
            if let Some(source) = self.source {
                res.source = Some(source);
                res.reload(app);
            } else if self.reload {
                res.reload(app);
            }
        });
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
enum Loading<T: Resource> {
    Path(#[derivative(Debug = "ignore")] AssetLoadingJob<Result<T::Loaded, ResourceError>>),
    Source(#[derivative(Debug = "ignore")] Job<Result<T::Loaded, ResourceError>>),
    Sync(#[derivative(Debug = "ignore")] T::Loaded),
}

#[derive(FromApp)]
struct ResManager<T: Resource>(PhantomData<T>);

impl<T> State for ResManager<T>
where
    T: Resource,
{
    fn update(&mut self, app: &mut App) {
        app.take::<Globals<Res<T>>, _>(|resources, app| {
            for res in resources.iter_mut() {
                res.update(app);
            }
        });
    }
}
