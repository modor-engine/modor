use crate::App;

/// A trait for creating a value using an [`App`].
///
/// # Examples
///
/// See [`modor`](crate).
pub trait FromApp: 'static {
    /// Creates the value from the [`App`].
    fn from_app(app: &mut App) -> Self;

    /// Creates the value with [`FromApp::from_app`] and applies `f`.
    fn from_app_with(app: &mut App, f: impl FnOnce(&mut Self, &mut App)) -> Self
    where
        Self: Sized,
    {
        let mut value = Self::from_app(app);
        f(&mut value, app);
        value
    }
}

impl<T> FromApp for T
where
    T: Default + 'static,
{
    fn from_app(_app: &mut App) -> Self {
        Self::default()
    }
}
