use std::any::Any;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::panic::{RefUnwindSafe, UnwindSafe};

/// The collision behavior that should happen between two type of collision groups.
///
/// # Examples
///
/// See [`PhysicsModule`](crate::PhysicsModule).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub enum CollisionType {
    /// No collision should happen.
    #[default]
    None,
    /// Collision should happen but it doesn't produce forces.
    Sensor,
    /// Collision should happen and it produces forces. This has currently the same effect as
    /// [`CollisionType::Sensor`](crate::CollisionType::Sensor).
    Impulse,
}

/// A trait for defining a collision group reference.
///
/// A collision group reference is generally an `enum` listing all the groups of collision
/// and describing which groups can collide together.<br>
/// This `enum` can then be assigned to a [`Collider2D`](crate::Collider2D).
///
/// # Examples
///
/// See [`PhysicsModule`](crate::PhysicsModule).
pub trait CollisionGroupRef:
    Any + Sync + Send + UnwindSafe + RefUnwindSafe + Clone + PartialEq + Eq + Hash + Debug
{
    /// Returns the collision type produced when the group collides with another group.
    ///
    /// It is not necessary to define the collision type in both ways.<br>
    /// For example, if groups `a` and `b` produces `CollisionType::Sensor`, and `b` and `a`
    /// produces `CollisionType::None`, when it considered that both cases produce
    /// `CollisionType::Sensor`.
    fn collision_type(&self, other: &Self) -> CollisionType;
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
