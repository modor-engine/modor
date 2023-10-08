use crate::Dynamics2D;
use modor::{Filter, With};
use modor_math::Vec2;

/// The positioning of a 2D entity.
///
/// # Related components
///
/// - [`Dynamics2D`]
///
/// # Example
///
/// See [`PhysicsModule`](crate::PhysicsModule).
#[non_exhaustive]
#[derive(Component, Clone, Debug)]
pub struct Transform2D {
    /// Position of the entity in world units.
    pub position: Vec2,
    /// Size of the entity in world units.
    pub size: Vec2,
    /// Rotation of the entity in radians.
    pub rotation: f32,
    pub(crate) old_position: Vec2,
    pub(crate) old_size: Vec2,
    pub(crate) old_rotation: f32,
}

#[systems]
impl Transform2D {
    /// Creates a new transform.
    #[inline]
    pub const fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            size: Vec2::ONE,
            rotation: 0.,
            old_position: Vec2::ZERO,
            old_size: Vec2::ONE,
            old_rotation: 0.,
        }
    }

    #[run_after(component(Dynamics2D))]
    fn update(&mut self, _filter: Filter<With<Dynamics2D>>) {
        self.old_position = self.position;
        self.old_size = self.size;
        self.old_rotation = self.rotation;
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::new()
    }
}
