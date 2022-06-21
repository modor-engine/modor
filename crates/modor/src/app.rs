use crate::storages::core::CoreStorage;
use crate::{Built, EntityMainComponent, Singleton};
use std::any::TypeId;

/// The entrypoint of the engine.
///
/// # Examples
///
/// ```rust
/// # #[macro_use]
/// # extern crate modor;
/// #
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
#[derive(Default)]
pub struct App {
    pub(crate) core: CoreStorage,
}

impl App {
    /// Creates a new empty `App`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Changes the number of threads used by the `App` during update.
    ///
    /// Update is only done in one thread if `count` is `0` or `1`,
    /// which is the default behavior.
    ///
    /// ## Platform-specific
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

    /// Returns the number of threads used by the `App` during update.
    #[must_use]
    pub fn thread_count(&self) -> u32 {
        self.core.systems().thread_count()
    }

    /// Runs `f` if the singleton of type `E` exists.
    pub fn run_for_singleton<E, F>(&mut self, f: F)
    where
        E: EntityMainComponent<Type = Singleton>,
        F: FnOnce(&mut E),
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

    /// Execute a `runner` that consumes the `App`.
    pub fn run<R>(self, runner: R)
    where
        R: FnOnce(Self),
    {
        runner(self);
    }
}
