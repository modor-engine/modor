use crate::from_app::FromApp;
use crate::App;
use derivative::Derivative;
use std::marker::PhantomData;

/// A value accessible from anywhere during [`App`] update.
///
/// # Examples
///
/// See [`modor`](crate).
pub trait Singleton: FromApp {
    /// Initializes the singleton.
    ///
    /// This method is run immediately after [`FromApp::from_app`].
    #[allow(unused_variables)]
    fn init(&mut self, app: &mut App) {}

    /// Update the singleton.
    ///
    /// This method is run once per [`App`] update.
    #[allow(unused_variables)]
    fn update(&mut self, app: &mut App) {}
}

/// A handle to access a [`Singleton`].
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
pub struct SingletonHandle<T> {
    index: usize,
    phantom: PhantomData<fn(T)>,
}

impl<T> SingletonHandle<T>
where
    T: Singleton,
{
    pub(crate) fn new(index: usize) -> Self {
        Self {
            index,
            phantom: PhantomData,
        }
    }

    /// Returns an immutable reference to the singleton.
    ///
    /// # Panics
    ///
    /// This will panic if the singleton is borrowed.
    pub fn get(self, app: &App) -> &T {
        app.get_from_index(self.index)
    }

    /// Returns a mutable reference to the singleton.
    ///
    /// # Panics
    ///
    /// This will panic if the singleton is borrowed.
    pub fn get_mut(self, app: &mut App) -> &mut T {
        app.get_mut_from_index(self.index)
    }

    /// Temporarily takes ownership of the singleton.
    ///
    /// This method is particularly useful to get mutable access to multiple singletons
    /// at the same time.
    ///
    /// # Panics
    ///
    /// This will panic if the singleton is borrowed.
    pub fn take(&mut self, app: &mut App, f: impl FnOnce(&mut T, &mut App))
    where
        T: Singleton,
    {
        app.take_from_index(self.index, f);
    }
}
