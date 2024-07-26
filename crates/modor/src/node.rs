use crate::{App, FromApp};

// TODO: rename to "singleton" everywhere

/// A trait for defining a root node accessible from anywhere during update.
///
/// [`RootNode`](macro@crate::RootNode) derive macro can be used in case the type implements
/// [`Default`].
///
/// # Examples
///
/// See [`modor`](crate).
pub trait RootNode: FromApp {
    /// Initializes the node.
    ///
    /// This method is called just after the node is created with [`FromApp`].
    #[allow(unused_variables)]
    fn init(&mut self, app: &mut App) {}

    /// Updates the node.
    ///
    /// This method is called once during each app update.
    #[allow(unused_variables)]
    fn update(&mut self, app: &mut App) {}
}
