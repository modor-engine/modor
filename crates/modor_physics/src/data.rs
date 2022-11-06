use std::ops::{Deref, DerefMut};

/// The index of a collision group.
///
/// # Examples
///
/// See [`PhysicsModule`](crate::PhysicsModule).
// This is an enum and not simplify a `usize` to make sure at compile time we don't go beyond the 32
// collision groups supported by Rapier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollisionGroupIndex {
    /// Index of collision group 0.
    Group0,
    /// Index of collision group 1.
    Group1,
    /// Index of collision group 2.
    Group2,
    /// Index of collision group 3.
    Group3,
    /// Index of collision group 4.
    Group4,
    /// Index of collision group 5.
    Group5,
    /// Index of collision group 6.
    Group6,
    /// Index of collision group 7.
    Group7,
    /// Index of collision group 8.
    Group8,
    /// Index of collision group 9.
    Group9,
    /// Index of collision group 10.
    Group10,
    /// Index of collision group 11.
    Group11,
    /// Index of collision group 12.
    Group12,
    /// Index of collision group 13.
    Group13,
    /// Index of collision group 14.
    Group14,
    /// Index of collision group 15.
    Group15,
    /// Index of collision group 16.
    Group16,
    /// Index of collision group 17.
    Group17,
    /// Index of collision group 18.
    Group18,
    /// Index of collision group 19.
    Group19,
    /// Index of collision group 20.
    Group20,
    /// Index of collision group 21.
    Group21,
    /// Index of collision group 22.
    Group22,
    /// Index of collision group 23.
    Group23,
    /// Index of collision group 24.
    Group24,
    /// Index of collision group 25.
    Group25,
    /// Index of collision group 26.
    Group26,
    /// Index of collision group 27.
    Group27,
    /// Index of collision group 28.
    Group28,
    /// Index of collision group 29.
    Group29,
    /// Index of collision group 30.
    Group30,
    /// Index of collision group 31.
    Group31,
}

impl CollisionGroupIndex {
    /// The list of all collision group indexes.
    pub const ALL: [Self; 32] = [
        Self::Group0,
        Self::Group1,
        Self::Group2,
        Self::Group3,
        Self::Group4,
        Self::Group5,
        Self::Group6,
        Self::Group7,
        Self::Group8,
        Self::Group9,
        Self::Group10,
        Self::Group11,
        Self::Group12,
        Self::Group13,
        Self::Group14,
        Self::Group15,
        Self::Group16,
        Self::Group17,
        Self::Group18,
        Self::Group19,
        Self::Group20,
        Self::Group21,
        Self::Group22,
        Self::Group23,
        Self::Group24,
        Self::Group25,
        Self::Group26,
        Self::Group27,
        Self::Group28,
        Self::Group29,
        Self::Group30,
        Self::Group31,
    ];
}

/// A collision layer.
///
/// Each group registered in a layer can collide with each other.
///
/// By default, entities of the same group cannot collide with each other.
/// This behavior can be changed by registering the group twice in the same layer.
///
/// # Examples
///
/// See [`PhysicsModule`](crate::PhysicsModule).
#[derive(Debug)]
pub struct CollisionLayer {
    pub(crate) groups: Vec<CollisionGroupIndex>,
}

impl CollisionLayer {
    /// Creates a new layer.
    #[must_use]
    pub const fn new(groups: Vec<CollisionGroupIndex>) -> Self {
        Self { groups }
    }
}

/// A property where changes are tracked internally.
#[derive(Debug, Default)]
pub struct PhysicsProperty<T> {
    value: T,
    is_changed: bool,
}

impl<T> PhysicsProperty<T> {
    pub(crate) const fn new(value: T) -> Self {
        Self {
            value,
            is_changed: true,
        }
    }

    pub(crate) fn replace(&mut self, value: T) {
        self.value = value;
    }

    pub(crate) fn consume_ref_if_changed(&mut self) -> Option<&T> {
        let changed = self.is_changed;
        self.is_changed = false;
        changed.then_some(&self.value)
    }

    pub(crate) fn consume_ref(&mut self) -> &T {
        self.is_changed = false;
        &self.value
    }
}

impl<T> Clone for PhysicsProperty<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            is_changed: true,
        }
    }
}

impl<T> Deref for PhysicsProperty<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for PhysicsProperty<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.is_changed = true;
        &mut self.value
    }
}
