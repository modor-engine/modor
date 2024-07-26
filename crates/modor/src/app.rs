use crate::{platform, RootNode};
use derivative::Derivative;
use fxhash::FxHashMap;
use log::{debug, Level};
use std::any;
use std::any::{Any, TypeId};
use std::marker::PhantomData;

/// The entrypoint of the engine.
///
/// # Examples
///
/// See [`modor`](crate).
#[derive(Debug)]
pub struct App {
    root_indexes: FxHashMap<TypeId, usize>,
    roots: Vec<RootNodeData>, // ensures deterministic update order
}

impl App {
    /// Creates a new app with an initial root node of type `T`.
    ///
    /// This also configures logging with a minimum `log_level` to display.
    ///
    /// # Platform-specific
    ///
    /// - Web: logging is initialized using the `console_log` crate and panic hook using the
    ///     `console_error_panic_hook` crate.
    /// - Other: logging is initialized using the `pretty_env_logger` crate.
    pub fn new<T>(log_level: Level) -> Self
    where
        T: RootNode,
    {
        platform::init_logging(log_level);
        debug!("Initialize app...");
        let mut app = Self {
            root_indexes: FxHashMap::default(),
            roots: vec![],
        };
        app.get_mut::<T>();
        debug!("App initialized");
        app
    }

    /// Update all root nodes registered in the app.
    ///
    /// [`RootNode::update`] method is called for each registered root node.
    ///
    /// Root nodes are updated in the order in which they are created.
    ///
    /// # Panics
    ///
    /// This will panic if any root node is already borrowed.
    pub fn update(&mut self) {
        debug!("Run update app...");
        for root_index in 0..self.roots.len() {
            let root = &mut self.roots[root_index];
            let mut value = root.value.take().expect("root node is already borrowed");
            let update_fn = root.update_fn;
            update_fn(&mut *value, self);
            self.roots[root_index].value = Some(value);
        }
        debug!("App updated");
    }

    /// Returns a handle to a root node.
    ///
    /// The root node is created using [`FromApp::from_app`](crate::FromApp::from_app)
    /// and [`RootNode::init`] if it doesn't exist.
    pub fn handle<T>(&mut self) -> RootNodeHandle<T>
    where
        T: RootNode,
    {
        RootNodeHandle {
            index: self.root_index_or_create::<T>(),
            phantom: PhantomData,
        }
    }

    /// Creates the root node of type `T` using [`FromApp::from_app`](crate::FromApp::from_app)
    /// and [`RootNode::init`] if it doesn't exist.
    pub fn create<T>(&mut self)
    where
        T: RootNode,
    {
        self.handle::<T>();
    }

    /// Returns a mutable reference to a root node.
    ///
    /// The root node is created using [`FromApp::from_app`](crate::FromApp::from_app)
    /// and [`RootNode::init`] if it doesn't exist.
    ///
    /// # Panics
    ///
    /// This will panic if root node `T` is already borrowed.
    pub fn get_mut<T>(&mut self) -> &mut T
    where
        T: RootNode,
    {
        let root_index = self.root_index_or_create::<T>();
        self.root_mut(root_index)
    }

    /// Borrows a root node without borrowing the app.
    ///
    /// The method returns the output of `f`.
    ///
    /// The root node is created using [`FromApp::from_app`](crate::FromApp::from_app)
    /// and [`RootNode::init`] if it doesn't exist.
    ///
    /// This method is useful when it is needed to have a mutable reference to multiple root nodes.
    ///
    /// # Panics
    ///
    /// This will panic if root node `T` is already borrowed.
    pub fn take<T, O>(&mut self, f: impl FnOnce(&mut T, &mut Self) -> O) -> O
    where
        T: RootNode,
    {
        let root_index = self.root_index_or_create::<T>();
        self.take_root(root_index, f)
    }

    #[allow(clippy::map_entry)]
    fn root_index_or_create<T>(&mut self) -> usize
    where
        T: RootNode,
    {
        let type_id = TypeId::of::<T>();
        if self.root_indexes.contains_key(&type_id) {
            self.root_indexes[&type_id]
        } else {
            debug!("Create root node `{}`...", any::type_name::<T>());
            let root = RootNodeData::new(T::from_app(self));
            debug!("Root node `{}` created", any::type_name::<T>());
            let index = self.roots.len();
            self.root_indexes.insert(type_id, index);
            self.roots.push(root);
            index
        }
    }

    fn root_mut<T>(&mut self, root_index: usize) -> &mut T
    where
        T: RootNode,
    {
        self.roots[root_index]
            .value
            .as_mut()
            .unwrap_or_else(|| panic!("root node `{}` already borrowed", any::type_name::<T>()))
            .downcast_mut::<T>()
            .expect("internal error: misconfigured root node")
    }

    fn take_root<T, O>(&mut self, root_index: usize, f: impl FnOnce(&mut T, &mut Self) -> O) -> O
    where
        T: RootNode,
    {
        let root = &mut self.roots[root_index];
        let mut value = root
            .value
            .take()
            .unwrap_or_else(|| panic!("root node `{}` already borrowed", any::type_name::<T>()));
        let value_ref = value
            .downcast_mut()
            .expect("internal error: misconfigured root node");
        let result = f(value_ref, self);
        self.roots[root_index].value = Some(value);
        result
    }
}

/// A handle to access a [`RootNode`].
#[derive(Derivative)]
#[derivative(
    Debug(bound = ""),
    Clone(bound = ""),
    Copy(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = ""),
    Hash(bound = "")
)]
pub struct RootNodeHandle<T> {
    index: usize,
    phantom: PhantomData<fn(T)>,
}

impl<T> RootNodeHandle<T>
where
    T: RootNode,
{
    /// Returns an immutable reference to the root node.
    ///
    /// # Panics
    ///
    /// This will panic if the root node is already borrowed.
    pub fn get(self, app: &App) -> &T {
        app.roots[self.index]
            .value
            .as_ref()
            .unwrap_or_else(|| panic!("root node `{}` already borrowed", any::type_name::<T>()))
            .downcast_ref::<T>()
            .expect("internal error: misconfigured root node")
    }

    /// Returns a mutable reference to the root node.
    ///
    /// # Panics
    ///
    /// This will panic if the root node is already borrowed.
    pub fn get_mut(self, app: &mut App) -> &mut T {
        app.root_mut(self.index)
    }

    /// Borrows a root node without borrowing the app.
    ///
    /// The method returns the output of `f`.
    ///
    /// This method is useful when it is needed to have a mutable reference to multiple root nodes.
    ///
    /// # Panics
    ///
    /// This will panic if the root node is already borrowed.
    pub fn take<O>(self, app: &mut App, f: impl FnOnce(&mut T, &mut App) -> O) -> O {
        app.take_root(self.index, f)
    }
}

#[derive(Debug)]
struct RootNodeData {
    value: Option<Box<dyn Any>>,
    update_fn: fn(&mut dyn Any, &mut App),
}

impl RootNodeData {
    fn new<T>(value: T) -> Self
    where
        T: RootNode,
    {
        Self {
            value: Some(Box::new(value)),
            update_fn: |value, app| {
                let value = value
                    .downcast_mut::<T>()
                    .expect("internal error: misconfigured singleton");
                T::update(value, app);
            },
        }
    }
}
