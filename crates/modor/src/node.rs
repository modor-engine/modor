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
    // TODO: add init()

    /// Updates the node.
    ///
    /// This method is called once during each app update.
    #[allow(unused_variables)]
    fn update(&mut self, app: &mut App) {}
}
