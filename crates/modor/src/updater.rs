use crate::Glob;

/// A trait for updating an object.
///
/// This trait can be implemented with the [`Updater`](macro@crate::Updater) derive macro.
pub trait Updater: Sized + 'static {
    /// The type use to perform the update.
    type Updater<'a>;

    /// Creates the updater.
    fn updater(&mut self) -> Self::Updater<'_>;
}

/// A trait for updating a glob.
///
/// This trait can be implemented with the [`GlobUpdater`](macro@crate::GlobUpdater) derive macro.
pub trait GlobUpdater: Sized + 'static {
    /// The type use to perform the update.
    type Updater<'a>;

    /// Creates the updater.
    fn updater(glob: &Glob<Self>) -> Self::Updater<'_>;
}

/// Updates a field.
///
/// `field` value is changed only if `new_value` is `Some`.
///
/// `is_updated` is set to `true` if `field` is different than `new_value` inner value.
///
/// # Examples
///
/// See [`Updater`](macro@crate::Updater) and [`GlobUpdater`](macro@crate::GlobUpdater).
pub fn update_field<U>(field: &mut U, new_value: Option<U>, is_updated: &mut bool)
where
    U: PartialEq,
{
    if let Some(new_value) = new_value {
        *is_updated |= field != &new_value;
        *field = new_value;
    }
}
