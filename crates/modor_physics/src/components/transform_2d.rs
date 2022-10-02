use crate::{Collider2D, PhysicsProperty, RelativeTransform2D};
use modor_math::Vec2;
use rapier2d::dynamics::{RigidBody, RigidBodyBuilder};
use rapier2d::geometry::{Collider, ColliderBuilder};
use rapier2d::na::Vector2;
use std::marker::PhantomData;

pub(crate) const ROOT_TRANSFORM: Transform2D = Transform2D::new();

/// The positioning of a 2D entity.
///
/// # Modor
///
/// - **Type**: component
/// - **Updated by**: [`PhysicsModule`](crate::PhysicsModule)
/// - **Updated during**: [`UpdatePhysicsAction`](crate::UpdatePhysicsAction)
/// - **Updated using**: [`RelativeTransform2D`](crate::RelativeTransform2D),
///     [`Dynamics2D`](crate::Dynamics2D), [`Collider2D`](crate::Collider2D),
///     [`DeltaTime`](crate::DeltaTime), [`Transform`](crate::Transform) of the parent
///
/// # Example
///
/// See [`PhysicsModule`](crate::PhysicsModule).
#[derive(Clone, Debug)]
pub struct Transform2D {
    /// Position of the entity in world units.
    pub position: PhysicsProperty<Vec2>,
    /// Size of the entity in world units.
    pub size: PhysicsProperty<Vec2>,
    /// Rotation of the entity in radians.
    pub rotation: PhysicsProperty<f32>,
    phantom: PhantomData<()>,
}

impl Transform2D {
    /// Creates a new transform.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            position: PhysicsProperty::new(Vec2::ZERO),
            size: PhysicsProperty::new(Vec2::ONE),
            rotation: PhysicsProperty::new(0.),
            phantom: PhantomData,
        }
    }

    /// Returns the transform with a different `position` in world units.
    ///
    /// Default value is `Vec2::ZERO`.
    #[must_use]
    #[inline]
    pub const fn with_position(mut self, position: Vec2) -> Self {
        self.position = PhysicsProperty::new(position);
        self
    }

    /// Returns the transform with a different `size` in world units.
    ///
    /// Default value is `Vec2::ONE`.
    #[must_use]
    #[inline]
    pub const fn with_size(mut self, size: Vec2) -> Self {
        self.size = PhysicsProperty::new(size);
        self
    }

    /// Returns the transform with a different `rotation` in radians.
    ///
    /// Default value is `0.0`.
    #[must_use]
    #[inline]
    pub const fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = PhysicsProperty::new(rotation);
        self
    }

    pub(crate) fn update_from_relative(
        &mut self,
        relative: &mut RelativeTransform2D,
        parent: &Self,
    ) {
        if let Some(relative_position) = relative.position {
            *self.position = relative_position
                .with_scale(*parent.size)
                .with_rotation(*parent.rotation)
                + *parent.position;
        }
        if let Some(relative_size) = relative.size {
            *self.size = parent.size.with_scale(relative_size);
        }
        if let Some(relative_rotation) = relative.rotation {
            *self.rotation = *parent.rotation + relative_rotation;
        }
    }

    pub(crate) fn update_from_body(&mut self, body: &RigidBody) {
        let position = body.translation();
        self.position.replace(Vec2::new(position.x, position.y));
        self.rotation.replace(body.rotation().angle());
    }

    pub(crate) fn update_resources(
        &mut self,
        body: &mut Option<&mut RigidBody>,
        rapier_collider: &mut Option<&mut Collider>,
        collider: Option<&&mut Collider2D>,
    ) {
        if let Some(&position) = self.position.consume_ref_if_changed() {
            if let Some(body) = body {
                body.set_translation(Vector2::new(position.x, position.y), true);
            }
            if let Some(rapier_collider) = rapier_collider {
                rapier_collider.set_translation(Vector2::new(position.x, position.y));
            }
        }
        if let Some(&rotation) = self.rotation.consume_ref_if_changed() {
            if let Some(body) = body {
                body.set_rotation(rotation, true);
            }
            if let Some(rapier_collider) = rapier_collider {
                rapier_collider.set_rotation(rotation);
            }
        }
        if let Some(&size) = self.size.consume_ref_if_changed() {
            if let (Some(rapier_collider), Some(collider)) = (rapier_collider, collider) {
                collider.update_collider(size, rapier_collider);
            }
        }
    }

    pub(crate) fn updated_body_builder(&mut self, builder: RigidBodyBuilder) -> RigidBodyBuilder {
        let position = self.position.consume_ref();
        builder
            .translation(Vector2::new(position.x, position.y))
            .rotation(*self.rotation.consume_ref())
    }

    pub(crate) fn updated_collider_builder(&mut self, builder: ColliderBuilder) -> ColliderBuilder {
        let position = self.position.consume_ref();
        builder
            .translation(Vector2::new(position.x, position.y))
            .rotation(*self.rotation.consume_ref())
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::new()
    }
}
