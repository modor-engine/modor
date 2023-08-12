use crate::ResKey;
use derivative::Derivative;
use fxhash::{FxHashMap, FxHashSet};
use modor::{Component, Entity, Query, SingleMut};
use modor_jobs::AssetLoadingError;
use std::any::Any;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::{any, fmt};

/// A registry that keeps track of resources of type `R` identified by a unique
/// [`ResKey`](ResKey).
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_resources::*;
/// #
/// type CounterRegistry = ResourceRegistry<Counter>;
///
/// const COUNTER1: ResKey<Counter> = ResKey::new("counter-1");
/// const COUNTER2: ResKey<Counter> = ResKey::new("counter-2");
/// const IGNORED_COUNTER: ResKey<Counter> = ResKey::new("ignored");
///
/// App::new()
///     .with_entity(CounterRegistry::default())
///     .with_entity(Counter::new(COUNTER1))
///     .with_entity(Counter::new(COUNTER2))
///     .with_entity(Counter::new(IGNORED_COUNTER))
///     .with_entity(TotalCount::default())
///     .update();
///
/// #[derive(Component)]
/// struct Counter {
///     count: u32,
///     key: ResKey<Counter>,
/// }
///
/// #[systems]
/// impl Counter {
///     fn new(key: ResKey<Counter>) -> Self {
///         Self {
///             count: 0,
///             key,
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
///     fn key(&self) -> ResKey<Counter> {
///         self.key
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
///         mut counters: Custom<ResourceAccessor<Counter>>,
///      ) {
///         self.count = 0;
///         if let Some(counter) = counters.get(COUNTER1, &counters) {
///             self.count += counter.count;
///         }
///         if let Some(counter) = counters.get(COUNTER2, &counters) {
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
    entity_ids: FxHashMap<ResKey<R>, usize>,
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
            let previous = self.entity_ids.insert(key, entity.id());
            trace!(
                "`{}` resource of type `{}` detected",
                key.label(),
                any::type_name::<R>()
            );
            if previous.is_some() {
                self.duplicated_keys.run(key, |t| {
                    error!("duplicated `{}` resource of type `{t}`", key.label());
                });
            }
            if let ResourceState::Error(error) = resource.state() {
                self.failed_keys.run(key, |t| {
                    error!(
                        "loading failed for `{}` resource of type `{t}`: {error}",
                        key.label()
                    );
                });
            }
        }
    }

    /// Returns the resource corresponding to the `key` if it exists and is in
    /// [`ResourceState::Loaded`](ResourceState::Loaded) state.
    pub fn get<'a>(&mut self, key: ResKey<R>, query: &'a Query<'_, &R>) -> Option<&'a R> {
        if let Some(resource) = self.entity_ids.get(&key).and_then(|&i| query.get(i)) {
            match resource.state() {
                ResourceState::NotLoaded => self.not_loaded_keys.run(key, |t| {
                    warn!(
                        "try to use not loaded `{}` resource of type `{t}`",
                        key.label()
                    );
                }),
                ResourceState::Loading => {
                    trace!(
                        "`{}` resource of type `{}` ignored as currently loading",
                        key.label(),
                        any::type_name::<R>()
                    );
                }
                ResourceState::Error(_) => {
                    trace!(
                        "`{}` resource of type `{}` ignored as loading failed",
                        key.label(),
                        any::type_name::<R>()
                    );
                }
                ResourceState::Loaded => return Some(resource),
            }
        } else {
            self.missing_keys.run(key, |t| {
                warn!(
                    "try to use not found `{}` resource of type `{t}`",
                    key.label()
                );
            });
        }
        None
    }
}

/// A trait for defining a resource.
///
/// # Examples
///
/// See [`ResourceRegistry`](ResourceRegistry).
pub trait Resource: Sized {
    /// Retrieves the key of the resource.
    fn key(&self) -> ResKey<Self>;

    /// Retrieves the state of the resource.
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

impl Display for ResourceLoadingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat(e) => write!(f, "invalid resource format: {e}"),
            Self::AssetLoadingError(e) => write!(f, "asset loading error: {e}"),
            Self::LoadingError(e) => write!(f, "resource loading error: {e}"),
        }
    }
}

impl Error for ResourceLoadingError {}

/// A system parameter to facilitate retrieval of resources.
///
/// # Examples
///
/// See [`ResourceRegistry`](ResourceRegistry).
#[derive(SystemParam)]
pub struct ResourceAccessor<'a, R>
where
    R: Component,
{
    registry: SingleMut<'a, 'static, ResourceRegistry<R>>,
    resources: Query<'a, &'static R>,
}

impl<R> ResourceAccessor<'_, R>
where
    R: Resource + Component,
{
    /// Returns the resource corresponding to the `key` if it exists and is in
    /// [`ResourceState::Loaded`](ResourceState::Loaded) state.
    pub fn get(&mut self, key: ResKey<R>) -> Option<&R> {
        self.registry.get_mut().get(key, &self.resources)
    }
}

// used to avoid log spam
#[derive(Derivative)]
#[derivative(Debug(bound = ""), Default(bound = ""))]
struct ResourceOnce<R> {
    keys: FxHashSet<ResKey<R>>,
}

impl<R> ResourceOnce<R> {
    fn run(&mut self, key: ResKey<R>, f: impl FnOnce(&str)) {
        if !self.keys.contains(&key) {
            self.keys.insert(key);
            f(any::type_name::<R>());
        }
    }
}

#[cfg(test)]
mod resource_loading_error_tests {
    use crate::ResourceLoadingError;
    use modor_jobs::AssetLoadingError;

    #[test]
    fn display_invalid_format_error() {
        let error = ResourceLoadingError::InvalidFormat("error message".into());
        let message = format!("{error}");
        assert_eq!(message, "invalid resource format: error message");
    }

    #[test]
    fn display_asset_loading_error() {
        let error = ResourceLoadingError::AssetLoadingError(AssetLoadingError::InvalidAssetPath);
        let message = format!("{error}");
        assert_eq!(message, "asset loading error: invalid asset path");
    }

    #[test]
    fn display_loading_error() {
        let error = ResourceLoadingError::LoadingError("error message".into());
        let message = format!("{error}");
        assert_eq!(message, "resource loading error: error message");
    }
}
