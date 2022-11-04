use crate::storages::core::CoreStorage;
use crate::{
    logging, system, Built, EntityFilter, EntityMainComponent, Singleton, SystemBuilder,
    SystemData, SystemInfo, UsizeRange,
};
use crate::{Entity, Query};
use std::any;
use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::panic;
use std::panic::RefUnwindSafe;

pub use log::LevelFilter;

/// The entrypoint of the engine.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// fn main() {
///     let mut app = App::new()
///         .with_thread_count(4)
///         .with_entity(Button::build("New game".into()))
///         .with_entity(Button::build("Settings".into()))
///         .with_entity(Button::build("Exit".into()));
///     app.update();
/// }
///
/// struct Button;
///
/// #[entity]
/// impl Button {
///     fn build(label: String) -> impl Built<Self> {
///         EntityBuilder::new(Self).with(label)
///     }
/// }
/// ```
///
/// [`App`](crate::App) can also be used for testing:
///
/// ```rust
/// # use modor::*;
/// #
/// struct Count(usize);
///
/// struct Counter;
///
/// #[entity]
/// impl Counter {
///     fn build() -> impl Built<Self> {
///         EntityBuilder::new(Self).with(Count(0))
///     }
///
///     #[run]
///     fn increment(count: &mut Count) {
///         count.0 += 1;
///     }
/// }
///
/// #[test]
/// fn test_counter() {
/// # }
/// # fn main() {
///     App::new()
///         .with_entity(Counter::build())
///         .assert::<With<Counter>>(1, |e| {
///             e.has(|c: &Count| assert_eq!(c.0, 0))
///                 .has_not::<usize>()
///                 .child_count(0)
///         })
///         .updated()
///         .assert::<With<Counter>>(1, |e| e.has(|c: &Count| assert_eq!(c.0, 1)))
///         .with_update::<With<Counter>, _>(|count: &mut Count| count.0 = 42)
///         .assert::<With<Counter>>(1, |e| e.has(|c: &Count| assert_eq!(c.0, 42)));
/// }
/// ```
#[derive(Default)]
pub struct App {
    pub(crate) core: CoreStorage,
}

impl App {
    /// Creates a new empty `App`.
    ///
    /// # Platform-specific
    ///
    /// - Web: logging is initialized using the `console_log` crate
    /// and panic hook using the `console_error_panic_hook` crate.
    /// - Other: logging is initialized using the `pretty_env_logger` crate.
    #[must_use]
    pub fn new() -> Self {
        logging::init();
        Self::default()
    }

    /// Set minimum log level.
    ///
    /// Default minimum log level is [`LevelFilter::Warn`](log::LevelFilter::Warn).
    #[must_use]
    pub fn with_log_level(self, level: LevelFilter) -> Self {
        log::set_max_level(level);
        self
    }

    /// Changes the number of threads used by the `App` during update.
    ///
    /// Update is only done in one thread if `count` is `0` or `1`,
    /// which is the default behavior.
    ///
    /// # Platform-specific
    ///
    /// - Web: update is done in one thread even if `count` if greater than `1`.
    #[must_use]
    pub fn with_thread_count(mut self, count: u32) -> Self {
        self.core.set_thread_count(count);
        self
    }

    /// Creates a new entity with main component of type `E`.
    #[must_use]
    pub fn with_entity<E, B>(mut self, entity: B) -> Self
    where
        E: EntityMainComponent,
        B: Built<E>,
    {
        entity.build(&mut self.core, None);
        self
    }

    /// Updates the component of type `C` of all entities that match `E` using `f`.
    ///
    /// If a matching entity does not have a component of type `C`, then the update is not
    /// performed for this entity.
    #[must_use]
    pub fn with_update<F, C>(mut self, mut f: impl FnMut(&mut C)) -> Self
    where
        F: EntityFilter,
        C: Any + Sync + Send,
    {
        let system = system!(|mut q: Query<'_, &mut C, F>| q.iter_mut().for_each(&mut f));
        let properties = (system.properties_fn)(&mut self.core);
        self.core.run_system_once(system.wrapper, properties);
        self
    }

    /// Runs all systems registered in the `App`.
    #[must_use]
    pub fn updated(mut self) -> Self {
        self.core.update();
        self
    }

    /// Runs all systems registered in the `App` until `f` returns `true` for the component of
    /// type `C` of any entity filtered with `F`.
    ///
    /// # Panics
    ///
    /// This will panic if `max_retry` is reached.
    #[must_use]
    pub fn updated_until_any<F, C>(
        mut self,
        max_retries: Option<usize>,
        mut f: impl FnMut(&C) -> bool,
    ) -> Self
    where
        F: EntityFilter,
        C: Any + Sync + Send,
    {
        let mut result = false;
        for i in 0.. {
            self.core.update();
            let system = system!(|entities: Query<'_, &C, F>| result = entities.iter().any(&mut f));
            let properties = (system.properties_fn)(&mut self.core);
            self.core.run_system_once(system.wrapper, properties);
            if result {
                break;
            }
            if let Some(max_retries) = max_retries {
                assert!(i < max_retries, "max number of retries reached");
            }
        }
        self
    }

    /// Runs all systems registered in the `App` until `f` returns `true` for the component of
    /// type `C` of all entity filtered with `F`.
    ///
    /// # Panics
    ///
    /// This will panic if `max_retry` is reached.
    #[must_use]
    pub fn updated_until_all<F, C>(
        mut self,
        max_retries: Option<usize>,
        mut f: impl FnMut(&C) -> bool,
    ) -> Self
    where
        F: EntityFilter,
        C: Any + Sync + Send,
    {
        let mut result = false;
        for i in 0.. {
            self.core.update();
            let system = system!(|entities: Query<'_, &C, F>| result = entities.iter().all(&mut f));
            let properties = (system.properties_fn)(&mut self.core);
            self.core.run_system_once(system.wrapper, properties);
            if result {
                break;
            }
            if let Some(max_retries) = max_retries {
                assert!(i < max_retries, "max number of retries reached");
            }
        }
        self
    }

    /// Asserts there are `count` entities matching `F` and run `f` on each of these entities.
    ///
    /// # Panics
    ///
    /// This will panic if `F` does not match `count` entities.
    #[allow(clippy::return_self_not_must_use)]
    pub fn assert<F>(
        mut self,
        count: impl UsizeRange,
        f: impl FnOnce(EntityAssertions<'_, F>) -> EntityAssertions<'_, F>,
    ) -> Self
    where
        F: EntityFilter,
    {
        let system = system!(|entities: Query<'_, (), F>| assert!(
            count.contains_value(entities.iter().len()),
            "assertion failed: {:?} entities matching {}, actual count: {}",
            count,
            any::type_name::<F>(),
            entities.iter().len(),
        ));
        let properties = (system.properties_fn)(&mut self.core);
        self.core.run_system_once(system.wrapper, properties);
        f(EntityAssertions {
            core: &mut self.core,
            any_mode: false,
            phantom: PhantomData,
        });
        self
    }

    /// Execute a `runner` that consumes the `App`.
    pub fn run<R>(self, runner: R)
    where
        R: FnOnce(Self),
    {
        runner(self);
    }

    /// Returns the number of threads used by the `App` during update.
    #[must_use]
    pub fn thread_count(&self) -> u32 {
        self.core.systems().thread_count()
    }

    /// Runs `f` if the singleton of type `E` exists.
    pub fn update_singleton<E>(&mut self, f: impl FnOnce(&mut E))
    where
        E: EntityMainComponent<Type = Singleton>,
    {
        let location = self
            .core
            .components()
            .type_idx(TypeId::of::<E>())
            .and_then(|t| self.core.components().singleton_location(t));
        if let Some(location) = location {
            f(&mut self.core.components().write_components()[location.idx][location.pos]);
        }
    }

    /// Runs all systems registered in the `App`.
    pub fn update(&mut self) {
        self.core.update();
    }
}

/// A utility for asserting on an entity matching `F` filter.
///
/// # Examples
///
/// See [`App`](crate::App).
pub struct EntityAssertions<'a, F> {
    core: &'a mut CoreStorage,
    any_mode: bool,
    phantom: PhantomData<F>,
}

#[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
impl<F> EntityAssertions<'_, F>
where
    F: EntityFilter,
{
    /// Configures the next assertions to pass in case they are true for any entity.
    ///
    /// By default, assertions must be true for all entities.
    pub fn any(mut self) -> Self {
        self.any_mode = true;
        self
    }

    /// Asserts the entity has a component of type `C` and run `f` on this component.
    ///
    /// # Panics
    ///
    /// This will panic if the entity does not have a component of type `C` or if `f` panics.
    ///
    /// # Platform-specific
    ///
    /// - Web: panics if [`any`](crate::EntityAssertions::any) has been previously called
    /// because internal call to [`catch_unwind`](std::panic::catch_unwind) is unsupported.
    pub fn has<C, A>(self, f: A) -> Self
    where
        C: Any + Sync + Send + RefUnwindSafe,
        A: Fn(&C) + RefUnwindSafe,
    {
        self.check_platform_for_catch_unwind();
        let any_mode = self.any_mode;
        self.run(system!(|entities: Query<'_, Option<&C>, F>| {
            assert!(
                if any_mode {
                    entities.iter().any(|c| c.is_some())
                } else {
                    entities.iter().all(|c| c.is_some())
                },
                "assertion failed: entities matching {} have component {}",
                any::type_name::<F>(),
                any::type_name::<C>(),
            );
            if any_mode {
                for component in entities.iter().flatten() {
                    if panic::catch_unwind(|| f(component)).is_ok() {
                        return;
                    }
                }
            }
            for component in entities.iter().flatten() {
                f(component);
            }
        }))
    }

    /// Asserts the entity has not a component of type `C`.
    ///
    /// # Panics
    ///
    /// This will panic if the entity has a component of type `C`.
    pub fn has_not<C>(self) -> Self
    where
        C: Any + Sync + Send,
    {
        let any_mode = self.any_mode;
        self.run(system!(|entities: Query<'_, Option<&C>, F>| {
            assert!(
                if any_mode {
                    entities.iter().any(|c| c.is_none())
                } else {
                    entities.iter().all(|c| c.is_none())
                },
                "assertion failed: entities matching {} have not component {}",
                any::type_name::<F>(),
                any::type_name::<C>(),
            );
        }))
    }

    /// Asserts the entity has `count` children.
    ///
    /// # Panics
    ///
    /// This will panic if the entity has not `count` children.
    pub fn child_count(self, count: impl UsizeRange) -> Self {
        let any_mode = self.any_mode;
        self.run(system!(|entities: Query<'_, Entity<'_>, F>| {
            assert!(
                if any_mode {
                    entities
                        .iter()
                        .any(|e| count.contains_value(e.children().len()))
                } else {
                    entities
                        .iter()
                        .all(|e| count.contains_value(e.children().len()))
                },
                "assertion failed: entities matching {} have {:?} children, actual count: {}",
                any::type_name::<F>(),
                count,
                entities
                    .iter()
                    .map(|e| e.children().len())
                    .find(|&c| !count.contains_value(c))
                    .expect("internal error: "),
            );
        }))
    }

    /// Asserts the entity has a parent matching `P`.
    ///
    /// # Panics
    ///
    /// This will panic if the entity parent does not match `P`.
    pub fn has_parent<P>(self) -> Self
    where
        P: EntityFilter,
    {
        let any_mode = self.any_mode;
        self.run(system!(
            |entities: Query<'_, Entity<'_>, F>, parents: Query<'_, Entity<'_>, P>| {
                let parent_ids: Vec<_> = parents.iter().map(Entity::id).collect();
                assert!(
                    if any_mode {
                        entities
                            .iter()
                            .any(|e| e.parent().map_or(false, |p| parent_ids.contains(&p.id())))
                    } else {
                        entities
                            .iter()
                            .all(|e| e.parent().map_or(false, |p| parent_ids.contains(&p.id())))
                    },
                    "assertion failed: entities matching {} have parent matching {}",
                    any::type_name::<F>(),
                    any::type_name::<P>(),
                );
            }
        ))
    }

    fn run<S>(self, system: SystemBuilder<S>) -> Self
    where
        S: FnMut(SystemData<'_>, SystemInfo<'_>),
    {
        let properties = (system.properties_fn)(self.core);
        self.core.run_system_once(system.wrapper, properties);
        self
    }

    // coverage: off (platform check)
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(clippy::unused_self)]
    fn check_platform_for_catch_unwind(&self) {}

    #[cfg(target_arch = "wasm32")]
    fn check_platform_for_catch_unwind(&self) {
        assert!(!self.any_mode, "not supported");
    }
    // coverage: on
}
