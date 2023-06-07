use crate::storages::core::CoreStorage;
use crate::{
    platform, system, utils, BuildableEntity, Component, EntityFilter, Filter, UsizeRange, World,
};
use crate::{Entity, Query};
use std::any;
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
///         .with_entity(button("New game"))
///         .with_entity(button("Settings"))
///         .with_entity(button("Exit"));
///     app.update();
/// }
///
/// #[derive(Component, NoSystem)]
/// struct Label(String);
///
/// #[derive(Component, NoSystem)]
/// struct Button;
///
/// fn button(label: &str) -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with(Button)
///         .with(Label(label.into()))
/// }
/// ```
///
/// See [`EntityBuilder`](crate::EntityBuilder) for details about how to create entities.
///
/// [`App`](App) can also be used for testing:
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Component)]
/// struct Count(usize);
///
/// #[systems]
/// impl Count {
///     #[run]
///     fn increment(count: &mut Count) {
///         count.0 += 1;
///     }
/// }
///
/// #[derive(Component, NoSystem)]
/// struct OtherComponent;
///
/// #[test]
/// fn test_counter() {
/// # }
/// # fn main() {
///     App::new()
///         .with_entity(Count(0))
///         .assert::<With<Count>>(1, |e| {
///             e.has(|c: &Count| assert_eq!(c.0, 0))
///                 .has_not::<OtherComponent>()
///                 .child_count(0)
///         })
///         .updated()
///         .assert::<With<Count>>(1, |e| e.has(|c: &Count| assert_eq!(c.0, 1)))
///         .with_update::<With<Count>, _>(|count: &mut Count| count.0 = 42)
///         .assert::<With<Count>>(1, |e| e.has(|c: &Count| assert_eq!(c.0, 42)));
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
    pub fn new() -> Self {
        utils::init_logging();
        Self::default()
    }

    /// Returns the number of threads used by the `App` during update.
    pub fn thread_count(&self) -> u32 {
        self.core.systems().thread_count()
    }

    /// Set minimum log level.
    ///
    /// Default minimum log level is [`LevelFilter::Warn`].
    pub fn with_log_level(self, level: LevelFilter) -> Self {
        log::set_max_level(level);
        info!("minimum log level set to '{level}'");
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
    pub fn with_thread_count(mut self, count: u32) -> Self {
        self.core.set_thread_count(count);
        let new_thread_count = self.core.systems().thread_count();
        info!("thread count set to {new_thread_count}");
        self
    }

    /// Creates a new entity.
    pub fn with_entity(mut self, entity: impl BuildableEntity) -> Self {
        entity.build(&mut self.core, None);
        self.core.delete_replaced_entities();
        self
    }

    // TODO: test
    // TODO: see where can be used instead of with_update
    /// Adds the component returned by `component_builder` to all entities matching `F` filter.
    ///
    /// If an entity already has a component of type `C`, it is overwritten.
    pub fn with_component<F, C>(mut self, mut component_builder: impl FnMut() -> C) -> Self
    where
        F: EntityFilter,
        C: Component,
    {
        self.core
            .run_system(system!(|e: Entity<'_>, mut w: World<'_>, _: Filter<F>| {
                w.add_component(e.id(), component_builder());
            }));
        self
    }

    // TODO: test
    /// Deletes all entities matching `F` filter.
    pub fn with_deleted_entities<F>(mut self) -> Self
    where
        F: EntityFilter,
    {
        self.core.run_system(system!(
            |e: Entity<'_>, mut w: World<'_>, _: Filter<F>| w.delete_entity(e.id())
        ));
        self
    }

    // TODO: test
    /// Deletes component of type `C` of each entity matching `F` filter.
    ///
    /// If a matching entity doesn't have a component of type `C`, nothing is done.
    pub fn with_deleted_components<F, C>(mut self) -> Self
    where
        F: EntityFilter,
        C: Component,
    {
        self.core.run_system(system!(
            |e: Entity<'_>, mut w: World<'_>, _: Filter<F>| w.delete_component::<C>(e.id())
        ));
        self
    }

    /// Updates the component of type `C` of all entities that match `E` using `f`.
    ///
    /// If a matching entity does not have a component of type `C`, then the update is not
    /// performed for this entity.
    pub fn with_update<F, C>(mut self, mut f: impl FnMut(&mut C)) -> Self
    where
        F: EntityFilter,
        C: Component,
    {
        self.core
            .run_system(system!(|c: &mut C, _: Filter<F>| f(c)));
        self
    }

    /// Runs all systems registered in the `App`.
    pub fn updated(mut self) -> Self {
        self.update();
        self
    }

    /// Runs all systems registered in the `App` until `f` returns `true` for the component of
    /// type `C` of any entity filtered with `F`.
    ///
    /// # Panics
    ///
    /// This will panic if `max_retry` is reached.
    pub fn updated_until_any<F, C>(
        mut self,
        max_retries: Option<usize>,
        mut f: impl FnMut(&C) -> bool,
    ) -> Self
    where
        F: EntityFilter,
        C: Component,
    {
        for i in 0.. {
            self.update();
            let mut result = false;
            self.core
                .run_system(system!(|c: &mut C, _: Filter<F>| result = result || f(c)));
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
    pub fn updated_until_all<F, C>(
        mut self,
        max_retries: Option<usize>,
        mut f: impl FnMut(&C) -> bool,
    ) -> Self
    where
        F: EntityFilter,
        C: Component,
    {
        for i in 0.. {
            self.update();
            let mut result = true;
            self.core
                .run_system(system!(|c: &mut C, _: Filter<F>| result = result && f(c)));
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
    pub fn assert<F>(
        mut self,
        count: impl UsizeRange,
        f: impl FnOnce(EntityAssertions<'_, F>) -> EntityAssertions<'_, F>,
    ) -> Self
    where
        F: EntityFilter,
    {
        self.assert_entity_count::<F>(count);
        f(EntityAssertions {
            core: &mut self.core,
            phantom: PhantomData,
        });
        self
    }

    /// Asserts there are `count` entities matching `F` and run `f` on each of these entities.
    ///
    /// In contrary to [`App::assert_any`](App::assert_any), the assertions will not fail if they
    /// are true for at least one filtered entity.
    ///
    /// # Panics
    ///
    /// This will panic if `F` does not match `count` entities.
    pub fn assert_any<F>(
        mut self,
        count: impl UsizeRange,
        f: impl FnOnce(EntityAnyAssertions<'_, F>) -> EntityAnyAssertions<'_, F>,
    ) -> Self
    where
        F: EntityFilter,
    {
        self.assert_entity_count::<F>(count);
        f(EntityAnyAssertions {
            core: &mut self.core,
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

    /// Apply `f` on all components of type `C`.
    pub fn update_components<C>(&mut self, mut f: impl FnMut(&mut C))
    where
        C: Component,
    {
        self.core.run_system(system!(&mut f));
    }

    /// Runs all systems registered in the `App`.
    pub fn update(&mut self) {
        debug!("update `App`...");
        self.core.update();
        debug!("`App` updated");
    }

    fn assert_entity_count<F>(&mut self, count: impl UsizeRange + Sized)
    where
        F: EntityFilter,
    {
        let mut entity_count = 0;
        self.core
            .run_system(system!(|_: Filter<F>| entity_count += 1));
        assert!(
            count.contains_value(entity_count),
            "assertion failed: {:?} entities matching {}, actual count: {}",
            count,
            any::type_name::<F>(),
            entity_count,
        );
    }
}

/// A utility for asserting on all entities matching `F` filter.
///
/// # Examples
///
/// See [`App`](App).
pub struct EntityAssertions<'a, F> {
    core: &'a mut CoreStorage,
    phantom: PhantomData<F>,
}

impl<'a, F> EntityAssertions<'a, F>
where
    F: EntityFilter,
{
    /// Asserts the entity has a component of type `C` and run `f` on this component.
    ///
    /// # Panics
    ///
    /// This will panic if the entity does not have a component of type `C` or if `f` panics.
    pub fn has<C, A>(self, f: A) -> Self
    where
        C: Component,
        A: Fn(&C),
    {
        let mut entity_count = 0;
        let mut component_count = 0;
        self.core.run_system(system!(|c: Option<&C>, _: Filter<F>| {
            entity_count += 1;
            if let Some(component) = c {
                component_count += 1;
                f(component);
            }
        }));
        assert_eq!(
            entity_count,
            component_count,
            "assertion failed: entities matching {} have component {}",
            any::type_name::<F>(),
            any::type_name::<C>(),
        );
        self
    }

    /// Asserts the entity has not a component of type `C`.
    ///
    /// # Panics
    ///
    /// This will panic if the entity has a component of type `C`.
    pub fn has_not<C>(self) -> Self
    where
        C: Component,
    {
        let mut component_count = 0;
        self.core.run_system(system!(|c: Option<&C>, _: Filter<F>| {
            if c.is_some() {
                component_count += 1;
            }
        }));
        assert_eq!(
            component_count,
            0,
            "assertion failed: entities matching {} have not component {}",
            any::type_name::<F>(),
            any::type_name::<C>(),
        );
        self
    }

    /// Asserts the entity has `count` children.
    ///
    /// # Panics
    ///
    /// This will panic if the entity has not `count` children.
    pub fn child_count(self, count: impl UsizeRange) -> Self {
        let mut entity_count = 0;
        let mut correct_entity_count = 0;
        self.core.run_system(system!(|e: Entity<'_>, _: Filter<F>| {
            entity_count += 1;
            if count.contains_value(e.children().len()) {
                correct_entity_count += 1;
            }
        }));
        assert_eq!(
            correct_entity_count,
            entity_count,
            "assertion failed: entities matching {} have {:?} children",
            any::type_name::<F>(),
            count,
        );
        self
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
        let mut entity_count = 0;
        let mut correct_entity_count = 0;
        self.core.run_system(system!(
            |e: Entity<'_>, _: Filter<F>, p: Query<'_, Filter<P>>| {
                entity_count += 1;
                if let Some(parent) = e.parent() {
                    if p.get(parent.id()).is_some() {
                        correct_entity_count += 1;
                    }
                }
            }
        ));
        assert_eq!(
            correct_entity_count,
            entity_count,
            "assertion failed: entities matching {} have parent matching {}",
            any::type_name::<F>(),
            any::type_name::<P>(),
        );
        self
    }
}

/// A utility for asserting on any entity matching `F` filter.
///
/// # Examples
///
/// See [`App`](App).
pub struct EntityAnyAssertions<'a, F> {
    core: &'a mut CoreStorage,
    phantom: PhantomData<F>,
}

impl<F> EntityAnyAssertions<'_, F>
where
    F: EntityFilter,
{
    /// Asserts the entity has a component of type `C` and run `f` on this component.
    ///
    /// # Panics
    ///
    /// This will panic if the entity does not have a component of type `C` or if `f` panics.
    ///
    /// The method will also panic for Web platform as [`catch_unwind`](panic::catch_unwind)
    /// is unsupported.
    pub fn has<C, A>(self, f: A) -> Self
    where
        C: Component + RefUnwindSafe,
        A: Fn(&C) + RefUnwindSafe,
    {
        platform::check_catch_unwind_availability();
        let mut component_count = 0;
        let mut error = None;
        let mut ok_count = 0;
        self.core.run_system(system!(|c: Option<&C>, _: Filter<F>| {
            if let Some(component) = c {
                component_count += 1;
                if let Err(unwind_error) = panic::catch_unwind(|| f(component)) {
                    error = Some(unwind_error);
                } else {
                    ok_count += 1;
                }
            }
        }));
        assert!(
            component_count > 0,
            "assertion failed: entities matching {} have component {}",
            any::type_name::<F>(),
            any::type_name::<C>(),
        );
        if let Some(error) = error {
            if ok_count == 0 {
                panic::resume_unwind(error);
            }
        }
        self
    }

    /// Asserts the entity has not a component of type `C`.
    ///
    /// # Panics
    ///
    /// This will panic if the entity has a component of type `C`.
    pub fn has_not<C>(self) -> Self
    where
        C: Component,
    {
        let mut missing_component_count = 0;
        self.core.run_system(system!(|c: Option<&C>, _: Filter<F>| {
            if c.is_none() {
                missing_component_count += 1;
            }
        }));
        assert!(
            missing_component_count > 0,
            "assertion failed: entities matching {} have not component {}",
            any::type_name::<F>(),
            any::type_name::<C>(),
        );
        self
    }

    /// Asserts the entity has `count` children.
    ///
    /// # Panics
    ///
    /// This will panic if the entity has not `count` children.
    pub fn child_count(self, count: impl UsizeRange) -> Self {
        let mut correct_entity_count = 0;
        self.core.run_system(system!(|e: Entity<'_>, _: Filter<F>| {
            if count.contains_value(e.children().len()) {
                correct_entity_count += 1;
            }
        }));
        assert!(
            correct_entity_count > 0,
            "assertion failed: entities matching {} have {:?} children",
            any::type_name::<F>(),
            count,
        );
        self
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
        let mut correct_entity_count = 0;
        self.core.run_system(system!(
            |e: Entity<'_>, _: Filter<F>, p: Query<'_, Filter<P>>| {
                if let Some(parent) = e.parent() {
                    if p.get(parent.id()).is_some() {
                        correct_entity_count += 1;
                    }
                }
            }
        ));
        assert!(
            correct_entity_count > 0,
            "assertion failed: entities matching {} have parent matching {}",
            any::type_name::<F>(),
            any::type_name::<P>(),
        );
        self
    }
}
