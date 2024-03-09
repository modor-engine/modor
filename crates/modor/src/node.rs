use crate::Context;
use std::collections::HashMap;
use std::ops::DerefMut;

/// A trait for defining a root node accessible from anywhere during update.
///
/// [`RootNode`](macro@crate::RootNode) derive macro can be used in case the type implements
/// [`Default`].
///
/// # Examples
///
/// See [`modor`](crate).
pub trait RootNode: 'static + Node {
    ///
    fn on_create(ctx: &mut Context<'_>) -> Self;
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
    fn on_enter(&mut self, ctx: &mut Context<'_>) {}

    /// Runs logic after the node is updated.
    #[inline]
    #[allow(unused_variables)]
    fn on_exit(&mut self, ctx: &mut Context<'_>) {}

    /// Runs node update.
    ///
    /// By default, the following methods are executed in order:
    /// - [`Node::on_enter`]
    /// - [`Visit::visit`]
    /// - [`Node::on_exit`]
    ///
    /// It shouldn't be necessary to override the default implementation of this method. It is
    /// recommended instead to update the above methods.
    #[inline]
    fn update(&mut self, ctx: &mut Context<'_>) {
        self.on_enter(ctx);
        self.visit(ctx);
        self.on_exit(ctx);
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
    fn visit(&mut self, ctx: &mut Context<'_>);
}

impl Node for Box<dyn Node> {}

impl Visit for Box<dyn Node> {
    #[inline]
    fn visit(&mut self, ctx: &mut Context<'_>) {
        self.deref_mut().update(ctx);
    }
}

impl<T> Node for Option<T> where T: Node {}

impl<T> Visit for Option<T>
where
    T: Node,
{
    #[inline]
    fn visit(&mut self, ctx: &mut Context<'_>) {
        if let Some(node) = self {
            node.update(ctx);
        }
    }
}

impl<T> Node for Vec<T> where T: Node {}

impl<T> Visit for Vec<T>
where
    T: Node,
{
    fn visit(&mut self, ctx: &mut Context<'_>) {
        for node in self {
            node.update(ctx);
        }
    }
}

impl<K, V, S> Node for HashMap<K, V, S> where V: Node {}

impl<K, V, S> Visit for HashMap<K, V, S>
where
    V: Node,
{
    #[inline]
    fn visit(&mut self, ctx: &mut Context<'_>) {
        for node in self.values_mut() {
            node.update(ctx);
        }
    }
}
