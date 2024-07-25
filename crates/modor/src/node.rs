use crate::App;

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

// TODO: merge with RootNode
/// A trait for defining a node that can be automatically updated.
///
/// [`Node`](macro@crate::Node) derive macro can be used in case no specific logic should be run for
/// the node itself.
///
/// # Examples
///
/// See [`modor`](crate).
pub trait Node {
    /// Runs node update.
    fn update(&mut self, app: &mut App);
}
