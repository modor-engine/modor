use fxhash::{FxHashMap, FxHashSet};
use log::{error, warn};
use modor::{Component, Entity, Query};
use modor_jobs::{AssetLoadingError, AssetLoadingJob, Job};
use std::any::Any;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::{any, fmt, mem};

// TODO: move in a dedicated crate

pub struct ResourceKey(Box<dyn DynResourceKey>);

impl ResourceKey {
    pub fn new<T>(value: T) -> Self
    where
        T: Any + Clone + Hash + PartialEq + Eq + Debug + Sync + Send + UnwindSafe + RefUnwindSafe,
    {
        Self(Box::new(value))
    }
}

impl Clone for ResourceKey {
    fn clone(&self) -> Self {
        Self(self.0.dyn_clone())
    }
}

impl Hash for ResourceKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.dyn_hash(state);
    }
}

impl PartialEq for ResourceKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.dyn_partial_eq(&*other.0)
    }
}

impl Eq for ResourceKey {}

impl Debug for ResourceKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.dyn_fmt(f)
    }
}

trait DynResourceKey: Sync + Send + UnwindSafe + RefUnwindSafe {
    fn as_any(&self) -> &dyn Any;

    fn dyn_clone(&self) -> Box<dyn DynResourceKey>;

    fn dyn_hash(&self, hasher: &mut dyn Hasher);

    fn dyn_partial_eq(&self, other: &dyn DynResourceKey) -> bool;

    fn dyn_fmt(&self, f: &mut Formatter<'_>) -> fmt::Result;
}

impl<T> DynResourceKey for T
where
    T: Any + Clone + Hash + PartialEq + Eq + Debug + Sync + Send + UnwindSafe + RefUnwindSafe,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_clone(&self) -> Box<dyn DynResourceKey> {
        Box::new(self.clone())
    }

    fn dyn_hash(&self, mut hasher: &mut dyn Hasher) {
        T::hash(self, &mut hasher);
    }

    fn dyn_partial_eq(&self, other: &dyn DynResourceKey) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .map_or(false, |o| self == o)
    }

    fn dyn_fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt(f)
    }
}

pub trait IntoResourceKey {
    fn into_key(self) -> ResourceKey;
}

impl<T> IntoResourceKey for T
where
    T: Any + Clone + Hash + PartialEq + Eq + Debug + Sync + Send + UnwindSafe + RefUnwindSafe,
{
    fn into_key(self) -> ResourceKey {
        ResourceKey::new(self)
    }
}

pub trait Resource: Sized {
    fn key(&self) -> &ResourceKey;

    fn state(&self) -> ResourceState<'_>;
}

/// The state of a resource.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ResourceState<'a> {
    /// The resource loading has not yet started.
    #[default]
    NotLoaded,
    /// The resource is loading.
    Loading,
    /// The resource is loaded.
    Loaded,
    /// The resource returned an error during its loading.
    Error(&'a ResourceLoadingError),
}

/// An error that occurs during resource loading.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResourceLoadingError {
    /// The resource format is unsupported.
    InvalidFormat(String),
    /// There was an error while retrieving the asset.
    AssetLoadingError(AssetLoadingError),
    /// There was an error while loading the resource.
    LoadingError(String),
}

#[allow(clippy::use_debug)]
impl Display for ResourceLoadingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat(e) => write!(f, "invalid resource format: {e:?}"),
            Self::AssetLoadingError(e) => write!(f, "asset loading error: {e}"),
            Self::LoadingError(e) => write!(f, "resource loading error: {e}"),
        }
    }
}

#[derive(SingletonComponent, Debug)]
pub struct ResourceRegistry<R>
where
    R: Any,
{
    entity_ids: FxHashMap<ResourceKey, usize>,
    duplicated_keys: ResourceOnce<R>,
    missing_keys: ResourceOnce<R>,
    not_loaded_keys: ResourceOnce<R>,
    failed_keys: ResourceOnce<R>,
}

impl<R> Default for ResourceRegistry<R>
where
    R: Any,
{
    fn default() -> Self {
        Self {
            entity_ids: FxHashMap::default(),
            duplicated_keys: ResourceOnce::default(),
            missing_keys: ResourceOnce::default(),
            not_loaded_keys: ResourceOnce::default(),
            failed_keys: ResourceOnce::default(),
        }
    }
}

#[systems]
impl<R> ResourceRegistry<R>
where
    R: Resource + Component,
{
    #[run]
    fn update(&mut self, resources: Query<'_, (&R, Entity<'_>)>) {
        self.entity_ids.clear();
        for (resource, entity) in resources.iter() {
            let key = resource.key();
            let previous = self.entity_ids.insert(key.clone(), entity.id());
            if previous.is_some() {
                self.duplicated_keys.run(key, |k, t| {
                    error!("duplicated `{k:?}` resource of type `{t}`");
                });
            }
        }
    }

    pub fn get<'a>(&mut self, key: &ResourceKey, query: &'a Query<'_, &R>) -> Option<&'a R> {
        if let Some(resource) = self.entity_ids.get(key).and_then(|&i| query.get(i)) {
            match resource.state() {
                ResourceState::NotLoaded => self.not_loaded_keys.run(key, |k, t| {
                    warn!("try to use not loaded `{k:?}` resource of type `{t}`");
                }),
                ResourceState::Error(error) => self.failed_keys.run(key, |k, t| {
                    error!("loading failed for `{k:?}` resource of type `{t}`: {error}");
                }),
                ResourceState::Loading => (),
                ResourceState::Loaded => return Some(resource),
            }
        } else {
            self.missing_keys.run(key, |k, t| {
                warn!("try to use not found `{k:?}` resource of type `{t}`");
            });
        }
        None
    }
}

// used to avoid log spam
#[derive(Debug)]
struct ResourceOnce<R> {
    keys: FxHashSet<ResourceKey>,
    phantom: PhantomData<fn(R)>,
}

impl<R> Default for ResourceOnce<R> {
    fn default() -> Self {
        Self {
            keys: FxHashSet::default(),
            phantom: PhantomData,
        }
    }
}

impl<R> ResourceOnce<R> {
    fn run(&mut self, key: &ResourceKey, f: impl FnOnce(&ResourceKey, &str)) {
        if !self.keys.contains(key) {
            self.keys.insert(key.clone());
            f(key, any::type_name::<R>());
        }
    }
}

pub trait Load<D>: Sized {
    fn load_from_file(data: Vec<u8>) -> Result<Self, ResourceLoadingError>;

    fn load_from_data(data: &D) -> Result<Self, ResourceLoadingError>;
}

#[derive(Debug)]
pub struct ResourceHandler<T, D> {
    source: ResourceSource<D>,
    state: ResourceHandlerState<T>,
    is_used: bool,
}

impl<T, D> ResourceHandler<T, D>
where
    T: Any + Send + Debug + Load<D>,
    D: Any + Send + Clone,
{
    pub fn new(source: ResourceSource<D>) -> Self {
        Self {
            source,
            state: ResourceHandlerState::NotLoaded,
            is_used: false,
        }
    }

    pub fn state(&self) -> ResourceState<'_> {
        if self.is_used {
            ResourceState::Loaded
        } else {
            match &self.state {
                ResourceHandlerState::NotLoaded | ResourceHandlerState::NotReloaded => {
                    ResourceState::NotLoaded
                }
                ResourceHandlerState::DataLoading(_)
                | ResourceHandlerState::PathLoading(_)
                | ResourceHandlerState::Loaded(_) => ResourceState::Loading,
                ResourceHandlerState::Error(error) => ResourceState::Error(error),
                ResourceHandlerState::Used => unreachable!(),
            }
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    pub fn resource(&mut self) -> Option<T> {
        let (state, resource) = match mem::take(&mut self.state) {
            ResourceHandlerState::Loaded(resource) => {
                self.is_used = true;
                (ResourceHandlerState::Used, Some(resource))
            }
            state => (state, None),
        };
        self.state = state;
        resource
    }

    pub fn set_source(&mut self, source: ResourceSource<D>) {
        self.source = source;
        self.state = ResourceHandlerState::NotReloaded;
    }

    pub fn reset(&mut self) {
        self.state = ResourceHandlerState::NotLoaded;
        self.is_used = false;
    }

    pub fn reload(&mut self) {
        self.state = ResourceHandlerState::NotReloaded;
    }

    pub fn update<R>(&mut self, key: &ResourceKey) {
        self.state = match mem::take(&mut self.state) {
            ResourceHandlerState::NotLoaded | ResourceHandlerState::NotReloaded => {
                self.start_loading()
            }
            ResourceHandlerState::DataLoading(job) => Self::check_data_loading_job::<R>(job, key),
            ResourceHandlerState::PathLoading(job) => Self::check_path_loading_job(job),
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
        key: &ResourceKey,
    ) -> ResourceHandlerState<T> {
        match job.try_poll() {
            Ok(Some(Ok(resource))) => ResourceHandlerState::Loaded(resource),
            Ok(Some(Err(error))) => ResourceHandlerState::Error(error),
            Ok(None) => ResourceHandlerState::DataLoading(job),
            Err(_) => ResourceHandlerState::Error(ResourceLoadingError::LoadingError(format!(
                "loading job panicked for `{:?}` resource of type {:?}",
                key,
                any::type_name::<R>()
            ))),
        }
    }

    fn check_path_loading_job(
        mut job: AssetLoadingJob<Result<T, ResourceLoadingError>>,
    ) -> ResourceHandlerState<T> {
        match job.try_poll() {
            Ok(Some(Ok(resource))) => ResourceHandlerState::Loaded(resource),
            Ok(Some(Err(error))) => ResourceHandlerState::Error(error),
            Ok(None) => ResourceHandlerState::PathLoading(job),
            Err(error) => {
                ResourceHandlerState::Error(ResourceLoadingError::AssetLoadingError(error))
            }
        }
    }
}

#[derive(Debug)]
pub enum ResourceSource<D> {
    SyncData(D),
    AsyncData(D),
    AsyncPath(String),
}

#[derive(Debug)]
enum ResourceHandlerState<T> {
    NotLoaded,
    NotReloaded,
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
