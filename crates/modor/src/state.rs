use crate::{App, FromApp};

/// A trait for defining a state accessible from anywhere during update.
///
/// [`State`](macro@crate::State) derive macro can be used in case the type implements
/// [`Default`].
///
/// # Examples
///
/// See [`modor`](crate).
pub trait State: FromApp {
    /// Initializes the state.
    ///
    /// This method is called just after the state is created with [`FromApp`].
    #[allow(unused_variables)]
    fn init(&mut self, app: &mut App) {}

    /// Updates the state.
    ///
    /// This method is called once during each app update.
    #[allow(unused_variables)]
    fn update(&mut self, app: &mut App) {}
}
