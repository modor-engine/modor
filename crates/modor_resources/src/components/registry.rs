use crate::ResourceKey;
use fxhash::{FxHashMap, FxHashSet};
use modor::{Component, Entity, Query};
use modor_jobs::AssetLoadingError;
use std::any::Any;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::{any, fmt};

/// A registry that keeps track of resources of type `R` identified by a unique
/// [`ResourceKey`](ResourceKey).
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_resources::*;
/// #
/// type CounterRegistry = ResourceRegistry<Counter>;
///
/// App::new()
///     .with_entity(CounterRegistry::default())
///     .with_entity(Counter::new(CounterKey::Counter1))
///     .with_entity(Counter::new(CounterKey::Counter2))
///     .with_entity(Counter::new(CounterKey::Ignored))
///     .with_entity(TotalCount::default())
///     .update();
///
/// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// enum CounterKey {
///     Counter1,
///     Counter2,
///     Ignored,
/// }
///
/// #[derive(Component)]
/// struct Counter {
///     count: u32,
///     key: ResourceKey,
/// }
///
/// #[systems]
/// impl Counter {
///     fn new(key: impl IntoResourceKey) -> Self {
///         Self {
///             count: 0,
///             key: key.into_key(),
///         }
///     }
///
///     #[run]
///     fn update(&mut self) {
///         self.count += 1;
///     }
/// }
///
/// impl Resource for Counter {
///     fn key(&self) -> &ResourceKey {
///         &self.key
///     }
///
///     fn state(&self) -> ResourceState<'_> {
///         ResourceState::Loaded
///     }
/// }
///
/// #[derive(SingletonComponent, Default)]
/// struct TotalCount {
///     count: u32
/// }
///
/// #[systems]
/// impl TotalCount {
///     #[run_after(component(CounterRegistry), component(Counter))]
///     fn update(
///         &mut self,
///         mut counter_registry: SingleMut<'_, CounterRegistry>,
///         counters: Query<'_, &Counter>
///      ) {
///         let counter1_key = CounterKey::Counter1.into_key();
///         let counter2_key = CounterKey::Counter2.into_key();
///         self.count = 0;
///         if let Some(counter) = counter_registry.get(&counter1_key, &counters) {
///             self.count += counter.count;
///         }
///         if let Some(counter) = counter_registry.get(&counter2_key, &counters) {
///             self.count += counter.count;
///         }
///     }
/// }
/// ```
///
/// See also [`ResourceHandler`](crate::ResourceHandler) for more advanced resource loading.
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
            trace!(
                "`{:?}` resource of type `{}` detected",
                key,
                any::type_name::<R>()
            );
            if previous.is_some() {
                self.duplicated_keys.run(key, |k, t| {
                    error!("duplicated `{k:?}` resource of type `{t}`");
                });
            }
            if let ResourceState::Error(error) = resource.state() {
                self.failed_keys.run(key, |k, t| {
                    error!("loading failed for `{k:?}` resource of type `{t}`: {error}");
                });
            }
        }
    }

    /// Returns the resource corresponding to the `key` if it exists and is in
    /// [`ResourceState::Loaded`](ResourceState::Loaded) state.
    pub fn get<'a>(&mut self, key: &ResourceKey, query: &'a Query<'_, &R>) -> Option<&'a R> {
        if let Some(resource) = self.entity_ids.get(key).and_then(|&i| query.get(i)) {
            match resource.state() {
                ResourceState::NotLoaded => self.not_loaded_keys.run(key, |k, t| {
                    warn!("try to use not loaded `{k:?}` resource of type `{t}`");
                }),
                ResourceState::Loading => {
                    trace!(
                        "`{key:?}` resource of type `{}` ignored as currently loading",
                        any::type_name::<R>()
                    );
                }
                ResourceState::Error(_) => {
                    trace!(
                        "`{key:?}` resource of type `{}` ignored as loading failed",
                        any::type_name::<R>()
                    );
                }
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

/// A trait to define a resource.
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
    /// The resource loading is in progress.
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
