use crate::App;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

/// A trait for defining a root node accessible from anywhere during update.
///
/// [`RootNode`](macro@crate::RootNode) derive macro can be used in case the type implements
/// [`Default`].
///
/// # Examples
///
/// See [`modor`](crate).
pub trait RootNode: 'static + Node {
    /// Creates the root node.
    ///
    /// Note that this method shouldn't be called manually to create the node.
    fn on_create(app: &mut App) -> Self;
}

/// A trait for defining a node that can be automatically updated.
///
/// [`Node`](macro@crate::Node) derive macro can be used in case no specific logic should be run for
/// the node itself.
///
/// # Examples
///
/// See [`modor`](crate).
pub trait Node: Visit {
    /// Runs logic before the node is updated.
    #[inline]
    #[allow(unused_variables)]
    fn on_enter(&mut self, app: &mut App) {}

    /// Runs logic after the node is updated.
    #[inline]
    #[allow(unused_variables)]
    fn on_exit(&mut self, app: &mut App) {}

    /// Runs node update.
    ///
    /// This method should be automatically called by [`Visit::visit`].
    ///
    /// By default, the following methods are executed in order:
    /// - [`Node::on_enter`]
    /// - [`Visit::visit`]
    /// - [`Node::on_exit`]
    ///
    /// It shouldn't be necessary to override the default implementation of this method. It is
    /// recommended instead to update the above methods.
    #[inline]
    fn update(&mut self, app: &mut App) {
        self.on_enter(app);
        self.visit(app);
        self.on_exit(app);
    }

    /// Converts the node into a [`Const<Self>`].
    ///
    /// This method runs [`Node::update`] before making the conversion.
    #[inline]
    fn into_const(mut self, app: &mut App) -> Const<Self>
    where
        Self: Sized,
    {
        self.update(app);
        Const { inner: self }
    }
}

/// A trait for defining the visit of inner nodes.
///
/// It is recommended to use the [`Visit`](macro@crate::Visit) derive macro to implement this trait.
///
/// # Examples
///
/// See [`modor`](crate).
pub trait Visit {
    /// Visits the inner nodes.
    fn visit(&mut self, app: &mut App);
}

impl Node for Box<dyn Node> {}

impl Visit for Box<dyn Node> {
    #[inline]
    fn visit(&mut self, app: &mut App) {
        self.deref_mut().update(app);
    }
}

impl<T> Node for Box<T> where T: Node {}

impl<T> Visit for Box<T>
where
    T: Node,
{
    #[inline]
    fn visit(&mut self, app: &mut App) {
        self.deref_mut().update(app);
    }
}

impl<T> Node for Option<T> where T: Node {}

impl<T> Visit for Option<T>
where
    T: Node,
{
    #[inline]
    fn visit(&mut self, app: &mut App) {
        if let Some(node) = self {
            node.update(app);
        }
    }
}

impl<T> Node for Vec<T> where T: Node {}

impl<T> Visit for Vec<T>
where
    T: Node,
{
    fn visit(&mut self, app: &mut App) {
        for node in self {
            node.update(app);
        }
    }
}

impl<K, V, S> Node for HashMap<K, V, S> where V: Node {}

impl<K, V, S> Visit for HashMap<K, V, S>
where
    V: Node,
{
    #[inline]
    fn visit(&mut self, app: &mut App) {
        for node in self.values_mut() {
            node.update(app);
        }
    }
}

/// A wrapper on a constant node.
///
/// This type is mainly used for optimization purpose for cases where the [`Node::update`] has no
/// effect because the node is never mutated.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Node, Visit)]
/// struct Root {
///     constant: Const<Value>,
/// }
///
/// impl RootNode for Root {
///     fn on_create(app: &mut App) -> Self {
///         Self {
///             constant: Value(42).into_const(app),
///         }
///     }
/// }
///
/// #[derive(Node, Visit)]
/// struct Value(u32);
/// ```
#[derive(Debug)]
pub struct Const<T> {
    inner: T,
}

impl<T> Deref for Const<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
