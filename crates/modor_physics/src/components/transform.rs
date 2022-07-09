use crate::components::relative_transform::RelativeTransform;
use modor_math::{Mat4, Quat, Vec3};
use std::marker::PhantomData;

pub(crate) const ROOT_TRANSFORM: Transform = Transform::new();

/// The positioning of an entity.
///
/// # Modor
///
/// - **Type**: component
/// - **Updated by**: [`PhysicsModule`](crate::PhysicsModule)
/// - **Updated during**: [`UpdatePhysicsAction`](crate::UpdatePhysicsAction)
/// - **Updated using**: [`RelativeTransform`](crate::RelativeTransform),
///     [`DynamicBody`](crate::DynamicBody), [`DeltaTime`](crate::DeltaTime),
///     [`Transform`](crate::Transform) of the parent,
///
/// # Example
///
/// See [`PhysicsModule`](crate::PhysicsModule).
#[derive(Clone, Debug)]
pub struct Transform {
    /// Position of the entity in world units.
    pub position: Vec3,
    /// Size of the entity in world units.
    pub size: Vec3,
    /// Rotation of the entity in radians.
    pub rotation: Quat,
    phantom: PhantomData<()>,
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

impl Transform {
    /// Creates a new transform.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            size: Vec3::ONE,
            rotation: Quat::ZERO,
            phantom: PhantomData,
        }
    }

    /// Returns the transform with a different `position`.
    #[must_use]
    #[inline]
    pub const fn with_position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    /// Returns the transform with a different `size`.
    #[must_use]
    #[inline]
    pub const fn with_size(mut self, size: Vec3) -> Self {
        self.size = size;
        self
    }

    /// Returns the transform with a different `rotation`.
    #[must_use]
    #[inline]
    pub const fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    /// Returns the transformation matrix.
    #[must_use]
    pub fn create_matrix(&self) -> Mat4 {
        Mat4::from_scale(self.size) * self.rotation.matrix() * Mat4::from_position(self.position)
    }

    pub(crate) fn update(&mut self, relative: &RelativeTransform, parent: &Self) {
        if let Some(relative_size) = relative.size {
            self.size = parent.size.with_scale(relative_size);
        }
        if let Some(relative_rotation) = relative.rotation {
            self.rotation = parent.rotation * relative_rotation;
        }
        if let Some(relative_position) = relative.position {
            self.position = parent.rotation.matrix() * relative_position.with_scale(parent.size)
                + parent.position;
        }
    }
}
