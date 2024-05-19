#![allow(
    clippy::non_canonical_clone_impl,
    clippy::non_canonical_partial_ord_impl
)] // warnings caused by Derivative

use crate::{platform, Node, RootNode};
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
        app.root::<T>();
        debug!("App initialized");
        app
    }

    /// Update all root nodes registered in the app.
    ///
    /// [`Node::update`] method is called for each registered root node.
    ///
    /// Root nodes are updated in the order in which they are created.
    pub fn update(&mut self) {
        debug!("Run update app...");
        for root_index in 0..self.roots.len() {
            let root = &mut self.roots[root_index];
            let mut value = root
                .value
                .take()
                .expect("internal error: root node already borrowed");
            let update_fn = root.update_fn;
            update_fn(&mut *value, &mut self.ctx());
            self.roots[root_index].value = Some(value);
        }
        debug!("App updated");
    }

    /// Returns an update context.
    ///
    /// This method is generally used for testing purpose.
    pub fn ctx(&mut self) -> Context<'_> {
        Context { app: self }
    }

    // TODO: allow direct get/get_mut
    /// Returns a mutable reference to a root node.
    ///
    /// The root node is created using [`RootNode::on_create`] if it doesn't exist.
    pub fn root<T>(&mut self) -> &mut T
    where
        T: RootNode,
    {
        let root_index = self.root_index_or_create::<T>();
        self.root_mut(root_index)
    }

    fn root_index_or_create<T>(&mut self) -> usize
    where
        T: RootNode,
    {
        let type_id = TypeId::of::<T>();
        if self.root_indexes.contains_key(&type_id) {
            self.root_indexes[&type_id]
        } else {
            self.create_root::<T>(type_id)
        }
    }

    fn create_root<T>(&mut self, type_id: TypeId) -> usize
    where
        T: RootNode,
    {
        debug!("Create root node `{}`...", any::type_name::<T>());
        let root = RootNodeData::new(T::on_create(&mut self.ctx()));
        debug!("Root node `{}` created", any::type_name::<T>());
        let index = self.roots.len();
        self.root_indexes.insert(type_id, index);
        self.roots.push(root);
        index
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
}

// If `App` was directly accessible during update, it would be possible to run `App::update`.
// As this is not wanted, `App` is wrapped in `Context` to limit the allowed operations.
/// The context accessible during node update.
#[derive(Debug)]
pub struct Context<'a> {
    app: &'a mut App,
}

impl Context<'_> {
    /// Returns a mutable reference to a root node.
    ///
    /// The root node is created using [`RootNode::on_create`] if it doesn't exist.
    ///
    /// # Panics
    ///
    /// This will panic if root node `T` is currently updated.
    pub fn root<T>(&mut self) -> RootNodeHandle<T>
    where
        T: RootNode,
    {
        RootNodeHandle {
            index: self.app.root_index_or_create::<T>(),
            phantom: PhantomData,
        }
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
    pub fn get<'a>(self, ctx: &'a Context<'_>) -> &'a T {
        ctx.app.roots[self.index]
            .value
            .as_ref()
            .unwrap_or_else(|| panic!("root node `{}` already borrowed", any::type_name::<T>()))
            .downcast_ref::<T>()
            .expect("internal error: misconfigured root node")
    }

    /// Returns an immutable reference to the root node.
    pub fn get_mut<'a>(self, ctx: &'a mut Context<'_>) -> &'a mut T {
        ctx.app.root_mut(self.index)
    }
}

#[derive(Debug)]
struct RootNodeData {
    value: Option<Box<dyn Any>>,
    update_fn: fn(&mut dyn Any, &mut Context<'_>),
}

impl RootNodeData {
    fn new<T>(value: T) -> Self
    where
        T: RootNode,
    {
        Self {
            value: Some(Box::new(value)),
            update_fn: Self::update_root::<T>,
        }
    }

    fn update_root<T>(value: &mut dyn Any, ctx: &mut Context<'_>)
    where
        T: RootNode,
    {
        Node::update(
            value
                .downcast_mut::<T>()
                .expect("internal error: misconfigured root node"),
            ctx,
        );
    }
}
