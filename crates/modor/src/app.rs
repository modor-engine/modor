use crate::storages::hierarchy::HierarchyStorage;
use crate::storages::object_ids::{ObjectIdStorage, ReservedObjectId};
use crate::storages::objects::ObjectStorage;
use crate::{
    platform, BuildContext, Context, DynId, Id, Object, ObjectResult, UnitResult, UpdateContext,
    UsizeRange,
};
use log::{debug, error, info, Level, LevelFilter};
use std::any;
use std::marker::PhantomData;
use std::sync::Once;

/// The entrypoint of the engine.
///
/// # Examples
///
/// See [`modor`](crate).
///
/// [`App`](App) can also be used for testing:
///
/// ```rust
/// # use modor::{App, Object, NoRole, UpdateContext};
/// #
/// struct Counter(usize);
///
/// impl Object for Counter {
///     type Role = NoRole;
///
///     fn update(&mut self, _ctx: &mut UpdateContext<'_>) -> modor::Result<()> {
///         self.0 += 1;
///         Ok(())
///     }
/// }
///
/// #[modor::test]
/// fn test_counter() {
/// # }
/// # fn main() {
///     App::new()
///         .create(|_| Counter(0))
///         .for_each(1, |counter: &mut Counter, _ctx| assert_eq!(counter.0, 0))
///         .update()
///         .for_each(1, |counter: &mut Counter, _ctx| assert_eq!(counter.0, 1));
/// }
/// ```
#[derive(Debug)]
pub struct App {
    pub(crate) objects: ObjectStorage,
    pub(crate) object_ids: ObjectIdStorage,
    pub(crate) hierarchy: HierarchyStorage,
}

impl Default for App {
    fn default() -> Self {
        const DEFAULT_LEVEL: Level = Level::Warn;
        static INIT: Once = Once::new();
        INIT.call_once(|| platform::init_logging(DEFAULT_LEVEL));
        Self {
            objects: ObjectStorage::default(),
            object_ids: ObjectIdStorage::default(),
            hierarchy: HierarchyStorage::default(),
        }
    }
}

impl App {
    /// Creates a new empty `App`.
    ///
    /// # Platform-specific
    ///
    /// - Web: logging is initialized using the `console_log` crate and panic hook using the
    ///     `console_error_panic_hook` crate.
    /// - Other: logging is initialized using the `pretty_env_logger` crate.
    pub fn new() -> Self {
        Self::default()
    }

    // coverage: off (logs not easily testable)
    /// Set minimum log level.
    ///
    /// Default minimum log level is [`LevelFilter::Warn`].
    pub fn set_log_level(&mut self, level: LevelFilter) -> &mut Self {
        log::set_max_level(level);
        info!("minimum log level set to '{level}'");
        self
    }
    // coverage: on

    /// Creates a new object.
    ///
    /// If the object is a singleton and already exists, then nothing happens.
    ///
    /// In case an error is raised during creation, then no object is created.
    pub fn create<T, R>(&mut self, builder: impl FnOnce(&mut BuildContext<'_>) -> R) -> &mut Self
    where
        T: Object,
        R: ObjectResult<Object = T>,
    {
        let id = self.object_ids.reserve();
        let _ = self.create_object_or_rollback(id, None, builder);
        self
    }

    /// Runs code for each object of type `T`.
    ///
    /// This method is generally used for testing purpose.
    ///
    /// # Panics
    ///
    /// This will panic if the actual number of objects of type `T` doesn't match `expected_count`
    /// or if an error is returned from `f`.
    pub fn for_each<T, R>(
        &mut self,
        expected_count: impl UsizeRange,
        mut f: impl FnMut(&mut T, &mut UpdateContext<'_>) -> R,
    ) -> &mut Self
    where
        T: Object,
        R: UnitResult,
    {
        let mut actions = Vec::new();
        let mut actual_count = 0;
        self.objects
            .lock(|all_objects, objects| {
                for (id, object) in objects.iter_mut_enumerated() {
                    let mut context = Context {
                        objects: all_objects,
                        object_ids: &mut self.object_ids,
                        actions: &mut actions,
                        self_id: Some(id.into()),
                        phantom: PhantomData,
                    };
                    let result = f(object, &mut context);
                    actual_count += 1;
                    result.into_result()?;
                }
                Ok(())
            })
            .expect("raised error");
        assert!(
            expected_count.contains_value(actual_count),
            "assertion failed: expected {:?} objects of type `{}`, {} objects found",
            expected_count,
            any::type_name::<T>(),
            actual_count,
        );
        for action in actions {
            let _ = self.run_action(action);
        }
        self
    }

    /// Runs [`Object::update`] for all created objects.
    pub fn update(&mut self) -> &mut Self {
        let mut actions = Vec::new();
        self.objects.update(&mut self.object_ids, &mut actions);
        for action in actions {
            let _ = self.run_action(action);
        }
        self
    }

    pub(crate) fn create_object_or_rollback<T, R>(
        &mut self,
        id: ReservedObjectId<T>,
        parent: Option<DynId>,
        builder: impl FnOnce(&mut BuildContext<'_>) -> R,
    ) -> crate::Result<()>
    where
        T: Object,
        R: ObjectResult<Object = T>,
    {
        let id = match id {
            ReservedObjectId::New(id) => id,
            ReservedObjectId::Existing(_) => {
                debug!(
                    "singleton of type `{}` not created as it already exists", // no-coverage
                    any::type_name::<T>()                                      // no-coverage
                );
                return Ok(());
            }
        };
        let result = self.create_object(id, parent, builder);
        if let Err(err) = &result {
            self.delete_object(id.into());
            error!("`{}` object not created: {err}", any::type_name::<T>());
        } else {
            debug!("`{}` object with ID {} created", any::type_name::<T>(), id);
        }
        result
    }

    fn create_object<T, R>(
        &mut self,
        id: Id<T>,
        parent: Option<DynId>,
        builder: impl FnOnce(&mut BuildContext<'_>) -> R,
    ) -> crate::Result<()>
    where
        T: Object,
        R: ObjectResult<Object = T>,
    {
        let mut actions = Vec::new();
        let mut context = BuildContext {
            objects: &mut self.objects,
            object_ids: &mut self.object_ids,
            actions: &mut actions,
            self_id: Some(id.into()),
            phantom: PhantomData,
        };
        // hierarchy updated before object is built to ensure a correct rollback in case of error
        self.hierarchy.add(id, parent);
        let object = builder(&mut context).into_result()?;
        self.objects.add_object(id, object);
        for action in actions {
            self.run_action(action)?;
        }
        Ok(())
    }

    fn delete_object(&mut self, id: DynId) {
        self.hierarchy.delete(id, &mut |id| {
            self.object_ids.delete(id);
            self.objects.delete_object(id);
        });
    }

    fn run_action(&mut self, action: Action) -> crate::Result<()> {
        match action {
            Action::ObjectDeletion(id) => {
                self.delete_object(id);
                Ok(())
            }
            Action::Other(action) => action(self),
        }
    }
}

pub(crate) enum Action {
    ObjectDeletion(DynId),
    Other(OtherAction),
}

pub(crate) type OtherAction = Box<dyn FnOnce(&mut App) -> crate::Result<()>>;
