use std::mem;

/// The update performed on a field.
///
/// This type is used for the fields of the updater type generated by the
/// [`Updater`](macro@crate::Updater) derive macro.
///
/// # Examples
///
/// See [`Updater`](macro@crate::Updater).
pub enum Update<'a, T> {
    /// The field is unchanged.
    None,
    /// The field value will be replaced by a new value.
    Value(T),
    /// The field value will be updated using a closure.
    Closure(Box<dyn FnOnce(&mut T) + 'a>),
}

impl<T> Default for Update<'_, T> {
    fn default() -> Self {
        Self::None
    }
}

impl<T> Update<'_, T> {
    /// Extracts the new field value.
    ///
    /// The value is returned in one of the following cases:
    /// - `self` is [`Update::Value`].
    /// - `self` is [`Update::Closure`].
    ///
    /// `f` is used to get the current field value.
    ///
    /// At the end of the method execution, `self` is [`Update::None`].
    pub fn take_value(&mut self, f: impl FnOnce() -> T) -> Option<T> {
        match mem::take(self) {
            Update::Value(value) => Some(value),
            Update::Closure(closure) => {
                let mut field = f();
                closure(&mut field);
                Some(field)
            }
            Update::None => None,
        }
    }

    /// Extracts the new field value if it has changed.
    ///
    /// The value is returned in one of the following cases:
    /// - `self` is [`Update::Value`] and the new value is different from the value returned by `f`.
    /// - `self` is [`Update::Closure`].
    ///
    /// `f` is used to get the current field value.
    ///
    /// At the end of the method execution, `self` is [`Update::None`].
    pub fn take_value_checked(&mut self, f: impl FnOnce() -> T) -> Option<T>
    where
        T: PartialEq,
    {
        match mem::take(self) {
            Update::Value(value) => (value != f()).then_some(value),
            Update::Closure(closure) => {
                let mut field = f();
                closure(&mut field);
                Some(field)
            }
            Update::None => None,
        }
    }

    /// Apply the update on a `field`.
    ///
    /// At the end of the method execution, `self` is [`Update::None`].
    pub fn apply(&mut self, field: &mut T) {
        match mem::take(self) {
            Update::Value(value) => *field = value,
            Update::Closure(closure) => closure(field),
            Update::None => {}
        }
    }

    /// Apply the update on a `field`, and returns whether the field's value has changed.
    ///
    /// `true` is returned in one of the following cases:
    /// - `self` is [`Update::Value`] and the new value is different from `field`.
    /// - `self` is [`Update::Closure`].
    ///
    /// At the end of the method execution, `self` is [`Update::None`].
    pub fn apply_checked(&mut self, field: &mut T) -> bool
    where
        T: PartialEq,
    {
        match mem::take(self) {
            Update::Value(value) => {
                if value == *field {
                    false
                } else {
                    *field = value;
                    true
                }
            }
            Update::Closure(closure) => {
                closure(field);
                true
            }
            Update::None => false,
        }
    }
}
