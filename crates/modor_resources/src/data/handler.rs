use crate::{ResKey, Resource, ResourceLoadingError, ResourceState};
use modor_jobs::{AssetLoadingJob, Job};
use std::any::Any;
use std::fmt::Debug;
use std::{any, mem};

/// A handler to load a resource.
///
/// # Examples
///
/// ```
/// # use modor::*;
/// # use modor_resources::*;
/// #
/// #[derive(Component)]
/// struct ContentSize {
///     key: ResKey<Self>,
///     handler: ResourceHandler<LoadedSize, String>,
///     size: Option<usize>,
/// }
///
/// #[systems]
/// impl ContentSize {
///     fn from_file(key: ResKey<Self>, path: impl Into<String>) -> Self {
///         Self {
///             key,
///             handler: ResourceHandler::new(ResourceSource::AsyncPath(path.into())),
///             size: None,
///         }
///     }
///
///     fn from_string(key: ResKey<Self>, string: impl Into<String>) -> Self {
///         Self {
///             key,
///             handler: ResourceHandler::new(ResourceSource::AsyncData(string.into())),
///             size: None,
///         }
///     }
///
///     #[run]
///     fn update(&mut self) {
///         self.handler.update::<Self>(self.key);
///         self.size = self.size.take().or_else(|| self.handler.resource().map(|s| s.0));
///     }
///
///     fn get(&self) -> Option<usize> {
///         self.size
///     }
/// }
///
/// impl Resource for ContentSize {
///     fn key(&self) -> ResKey<Self> {
///         self.key
///     }
///
///     fn state(&self) -> ResourceState<'_> {
///         if self.size.is_some() {
///             ResourceState::Loaded
///         } else {
///             self.handler.state()
///         }
///     }
/// }
///
/// #[derive(Debug)]
/// struct LoadedSize(usize);
///
/// impl Load<String> for LoadedSize {
///     fn load_from_file(data: Vec<u8>) -> Result<Self, ResourceLoadingError> {
///         Ok(Self(data.len()))
///     }
///
///     fn load_from_data(data: &String) -> Result<Self, ResourceLoadingError> {
///         Ok(Self(data.len()))
///     }
/// }
/// ```
#[derive(Debug)]
pub struct ResourceHandler<T, D> {
    source: ResourceSource<D>,
    state: ResourceHandlerState<T>,
}

impl<T, D> ResourceHandler<T, D>
where
    T: Any + Send + Debug + Load<D>,
    D: Any + Send + Clone,
{
    /// Creates a new handler to load a resource of type `T` from a `source`.
    pub fn new(source: ResourceSource<D>) -> Self {
        Self {
            source,
            state: ResourceHandlerState::NotLoaded,
        }
    }

    /// Returns the state of the resource loading.
    pub fn state(&self) -> ResourceState<'_> {
        match &self.state {
            ResourceHandlerState::NotLoaded => ResourceState::NotLoaded,
            ResourceHandlerState::DataLoading(_)
            | ResourceHandlerState::PathLoading(_)
            | ResourceHandlerState::Loaded(_) => ResourceState::Loading,
            ResourceHandlerState::Error(error) => ResourceState::Error(error),
            ResourceHandlerState::Used => ResourceState::Loaded,
        }
    }

    /// Moves the resource out of the handler if loaded and not yet moved.
    #[allow(clippy::wildcard_enum_match_arm)]
    pub fn resource(&mut self) -> Option<T> {
        let (state, resource) = match mem::take(&mut self.state) {
            ResourceHandlerState::Loaded(resource) => (ResourceHandlerState::Used, Some(resource)),
            state => (state, None),
        };
        self.state = state;
        resource
    }

    /// Sets the `source` and start reset the handler.
    pub fn set_source(&mut self, source: ResourceSource<D>) {
        self.source = source;
        self.reload();
    }

    /// Restarts the resource loading.
    pub fn reload(&mut self) {
        self.state = ResourceHandlerState::NotLoaded;
    }

    /// Updates the state of the handler.
    ///
    /// The resource `key` and type `R` are only used for logging.
    ///
    /// Loading is performed only if this method is called. It is necessary to call this method
    /// multiple times until loading is finished.
    pub fn update<R>(&mut self, key: ResKey<R>)
    where
        R: Resource,
    {
        self.state = match mem::take(&mut self.state) {
            ResourceHandlerState::NotLoaded => self.start_loading(),
            ResourceHandlerState::DataLoading(job) => Self::check_data_loading_job(job, key),
            ResourceHandlerState::PathLoading(job) => Self::check_path_loading_job(job, key),
            state @ (ResourceHandlerState::Loaded(_)
            | ResourceHandlerState::Used
            | ResourceHandlerState::Error(_)) => state,
        };
    }

    fn start_loading(&self) -> ResourceHandlerState<T> {
        match &self.source {
            ResourceSource::SyncData(data) => match T::load_from_data(data) {
                Ok(resource) => ResourceHandlerState::Loaded(resource),
                Err(error) => ResourceHandlerState::Error(error),
            },
            ResourceSource::AsyncData(data) => {
                let data = data.clone();
                ResourceHandlerState::DataLoading(Job::new(async move { T::load_from_data(&data) }))
            }
            ResourceSource::AsyncPath(path) => {
                ResourceHandlerState::PathLoading(AssetLoadingJob::new(path, move |d| async move {
                    T::load_from_file(d)
                }))
            }
        }
    }

    fn check_data_loading_job<R>(
        mut job: Job<Result<T, ResourceLoadingError>>,
        key: ResKey<R>,
    ) -> ResourceHandlerState<T> {
        match job.try_poll() {
            Ok(Some(Ok(resource))) => ResourceHandlerState::Loaded(resource),
            Ok(Some(Err(error))) => ResourceHandlerState::Error(error),
            Ok(None) => ResourceHandlerState::DataLoading(job),
            Err(_) => {
                let error = format!(
                    "loading job panicked for `{}` resource of type `{}`",
                    key.label(),
                    any::type_name::<R>()
                );
                error!("{error}");
                ResourceHandlerState::Error(ResourceLoadingError::LoadingError(error))
            }
        }
    }

    fn check_path_loading_job<R>(
        mut job: AssetLoadingJob<Result<T, ResourceLoadingError>>,
        key: ResKey<R>,
    ) -> ResourceHandlerState<T> {
        match job.try_poll() {
            Ok(Some(Ok(resource))) => ResourceHandlerState::Loaded(resource),
            Ok(Some(Err(error))) => ResourceHandlerState::Error(error),
            Ok(None) => ResourceHandlerState::PathLoading(job),
            Err(error) => {
                error!(
                    "loading from path failed for `{}` resource of type `{}`: {error}",
                    key.label(),
                    any::type_name::<R>()
                );
                ResourceHandlerState::Error(ResourceLoadingError::AssetLoadingError(error))
            }
        }
    }
}

/// A trait for defining a loadable resource.
///
/// # Examples
///
/// See [`ResourceHandler`](ResourceHandler).
pub trait Load<D>: Sized {
    /// Loads the resource from raw file `data`.
    ///
    /// # Errors
    ///
    /// An error is returned if the resource cannot be loaded.
    fn load_from_file(data: Vec<u8>) -> Result<Self, ResourceLoadingError>;

    /// Loads the resource from custom data of type `D`.
    ///
    /// # Errors
    ///
    /// An error is returned if the resource cannot be loaded.
    fn load_from_data(data: &D) -> Result<Self, ResourceLoadingError>;
}

/// The source of a resource.
///
/// # Examples
///
/// See [`ResourceHandler`](ResourceHandler).
#[derive(Debug)]
pub enum ResourceSource<D> {
    /// Custom data of type `D` that should load the resource synchronously.
    SyncData(D),
    /// Custom data of type `D` that should load the resource asynchronously.
    AsyncData(D),
    /// File path where the resource should be retrieved and loaded asynchronously.
    ///
    /// # Platform-specific
    ///
    /// Given `path` the `String` path passed to the variant:
    /// - Web: HTTP GET call is performed to retrieve the file from URL
    /// `{current_browser_url}/assets/{path}`.
    /// - Android: the file is retrieved using the Android
    /// [`AssetManager`](https://developer.android.com/reference/android/content/res/AssetManager).
    /// - Other: if `CARGO_MANIFEST_DIR` environment variable is set (this is the case if the
    /// application is run using a `cargo` command), then the file is retrieved from path
    /// `{CARGO_MANIFEST_DIR}/assets/{path}`. Else, the file path is
    /// `{executable_folder_path}/assets/{path}`.
    AsyncPath(String),
}

#[derive(Debug)]
enum ResourceHandlerState<T> {
    NotLoaded,
    DataLoading(Job<Result<T, ResourceLoadingError>>),
    PathLoading(AssetLoadingJob<Result<T, ResourceLoadingError>>),
    Loaded(T),
    Used,
    Error(ResourceLoadingError),
}

impl<T> Default for ResourceHandlerState<T> {
    fn default() -> Self {
        Self::NotLoaded
    }
}
