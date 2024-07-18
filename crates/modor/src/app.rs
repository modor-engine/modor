use crate::singleton::SingletonHandle;
use crate::{platform, Singleton};
use fxhash::FxHashMap;
use log::{debug, Level};
use std::any;
use std::any::{Any, TypeId};

/// The entrypoint of the engine.
///
/// # Examples
///
/// See [`modor`](crate).
#[derive(Debug)]
pub struct App {
    singleton_indexes: FxHashMap<TypeId, usize>,
    singletons: Vec<SingletonData>,
}

impl App {
    /// Creates a new app.
    ///
    /// This configures logging with a minimum `log_level` to display.
    ///
    /// # Platform-specific
    ///
    /// - Web: logging is initialized using the `console_log` crate and panic hook using the
    ///     `console_error_panic_hook` crate.
    /// - Other: logging is initialized using the `pretty_env_logger` crate.
    pub fn new(log_level: Level) -> Self {
        platform::init_logging(log_level);
        debug!("App initialized");
        Self {
            singleton_indexes: FxHashMap::default(),
            singletons: vec![],
        }
    }

    /// Update all singletons registered in the app.
    ///
    /// [`Singleton::update`] method is called for each registered singleton.
    ///
    /// Singletons are updated in the order in which they are created.
    ///
    /// # Panics
    ///
    /// This will panic if any singleton is borrowed.
    pub fn update(&mut self) {
        debug!("Run update app...");
        for singleton_index in 0..self.singletons.len() {
            let singleton = &mut self.singletons[singleton_index];
            let mut value = singleton
                .value
                .take()
                .expect("internal error: singleton already borrowed");
            (singleton.update_fn)(&mut *value, self);
            self.singletons[singleton_index].value = Some(value);
        }
        debug!("App updated");
    }

    /// Creates a singleton if it doesn't exist.
    ///
    /// The singleton is created with
    /// [`FromApp::from_app`](crate::from_app::FromApp::from_app) and [`Singleton::init`].
    ///
    /// # Panics
    ///
    /// This will panic if the singleton is borrowed.
    pub fn create<T>(&mut self)
    where
        T: Singleton,
    {
        self.index_or_create::<T>();
    }

    /// Returns a handle to a singleton.
    ///
    /// If it doesn't exist, the singleton is created using
    /// [`FromApp::from_app`](crate::from_app::FromApp::from_app) and [`Singleton::init`].
    pub fn handle<T>(&mut self) -> SingletonHandle<T>
    where
        T: Singleton,
    {
        SingletonHandle::new(self.index_or_create::<T>())
    }

    /// Returns a mutable reference to a singleton.
    ///
    /// If it doesn't exist, the singleton is created using
    /// [`FromApp::from_app`](crate::from_app::FromApp::from_app) and [`Singleton::init`].
    ///
    /// # Panics
    ///
    /// This will panic if the singleton is borrowed.
    pub fn get_mut<T>(&mut self) -> &mut T
    where
        T: Singleton,
    {
        self.handle().get_mut(self)
    }

    /// Temporarily takes ownership of a singleton.
    ///
    /// This method is particularly useful to get mutable access to multiple singletons
    /// at the same time.
    ///
    /// # Panics
    ///
    /// This will panic if the singleton is borrowed.
    pub fn take<T>(&mut self, f: impl FnOnce(&mut T, &mut Self))
    where
        T: Singleton,
    {
        self.handle().take(self, f);
    }

    pub(crate) fn get_from_index<T>(&self, index: usize) -> &T
    where
        T: Singleton,
    {
        self.singletons[index]
            .value
            .as_ref()
            .unwrap_or_else(|| panic!("singleton `{}` already borrowed", any::type_name::<T>()))
            .downcast_ref::<T>()
            .expect("internal error: misconfigured singleton")
    }

    pub(crate) fn get_mut_from_index<T>(&mut self, index: usize) -> &mut T
    where
        T: Singleton,
    {
        self.singletons[index]
            .value
            .as_mut()
            .unwrap_or_else(|| panic!("singleton `{}` already borrowed", any::type_name::<T>()))
            .downcast_mut::<T>()
            .expect("internal error: misconfigured singleton")
    }

    pub(crate) fn take_from_index<T>(&mut self, index: usize, f: impl FnOnce(&mut T, &mut Self))
    where
        T: Singleton,
    {
        let mut singleton = self.singletons[index]
            .value
            .take()
            .unwrap_or_else(|| panic!("singleton `{}` already borrowed", any::type_name::<T>()));
        let singleton_ref = singleton
            .downcast_mut::<T>()
            .expect("internal error: misconfigured singleton");
        f(singleton_ref, self);
        self.singletons[index].value = Some(singleton);
    }

    fn index_or_create<T>(&mut self) -> usize
    where
        T: Singleton,
    {
        let type_id = TypeId::of::<T>();
        let index = *self
            .singleton_indexes
            .entry(type_id)
            .or_insert_with(|| self.singletons.len());
        if index == self.singletons.len() {
            debug!("Create singleton `{}`...", any::type_name::<T>());
            // reserve slot in case other singleton is creating during T::from_app_with
            self.singletons.push(SingletonData::new::<T>());
            let singleton = T::from_app_with(self, T::init);
            self.singleton_indexes.insert(type_id, index);
            self.singletons[index].value = Some(Box::new(singleton));
            debug!("Singleton `{}` created", any::type_name::<T>());
        }
        index
    }
}

#[derive(Debug)]
struct SingletonData {
    value: Option<Box<dyn Any>>,
    update_fn: fn(&mut dyn Any, &mut App),
}

impl SingletonData {
    fn new<T>() -> Self
    where
        T: Singleton,
    {
        Self {
            value: None,
            update_fn: |value, app| {
                let value = value
                    .downcast_mut::<T>()
                    .expect("internal error: misconfigured singleton");
                Singleton::update(value, app);
            },
        }
    }
}
