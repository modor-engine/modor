use crate::App;

// TODO: rename to "singleton" everywhere

/// A trait for defining a root node accessible from anywhere during update.
///
/// [`RootNode`](macro@crate::RootNode) derive macro can be used in case the type implements
/// [`Default`].
///
/// # Examples
///
/// See [`modor`](crate).
pub trait RootNode: 'static {
    // TODO: replace by FromApp
    /// Creates the root node.
    ///
    /// Note that this method shouldn't be called manually to create the node.
    fn on_create(app: &mut App) -> Self;

    // TODO: add init()

    /// Updates the node.
    ///
    /// This method is called once during each app update.
    #[allow(unused_variables)]
    fn update(&mut self, app: &mut App) {}
}
