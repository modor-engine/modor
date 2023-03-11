use crate::PhysicsProperty;
use modor_math::Vec2;
use rapier2d::dynamics::{RigidBody, RigidBodyBuilder, RigidBodyHandle};
use rapier2d::na::Vector2;

/// The dynamics properties of a 2D entity.
///
/// This component has an effect only if the entity has also a component of type
/// [`Transform2D`](crate::Transform2D).
///
/// # Example
///
/// See [`PhysicsModule`](crate::PhysicsModule).
#[derive(Debug, Clone, Component, NoSystem)]
pub struct Dynamics2D {
    /// Linear velocity of the entity in world units per second.
    pub velocity: PhysicsProperty<Vec2>,
    /// Angular velocity of the entity in radians per second.
    pub angular_velocity: PhysicsProperty<f32>,
    pub(crate) handle: Option<RigidBodyHandle>,
}

impl Dynamics2D {
    /// Creates a new body.
    #[inline]
    pub const fn new() -> Self {
        Self {
            velocity: PhysicsProperty::new(Vec2::ZERO),
            angular_velocity: PhysicsProperty::new(0.),
            handle: None,
        }
    }

    /// Returns the dynamics with a different `velocity` in world units per second.
    ///
    /// Default value is `Vec2::ZERO`.
    #[inline]
    pub const fn with_velocity(mut self, velocity: Vec2) -> Self {
        self.velocity = PhysicsProperty::new(velocity);
        self
    }

    /// Returns the dynamics with a different `angular_velocity` in radians per second.
    ///
    /// Default value is `0.0`.
    #[inline]
    pub const fn with_angular_velocity(mut self, angular_velocity: f32) -> Self {
        self.angular_velocity = PhysicsProperty::new(angular_velocity);
        self
    }

    pub(crate) fn update_from_body(&mut self, body: &RigidBody) {
        let velocity = body.linvel();
        self.velocity.replace(Vec2::new(velocity.x, velocity.y));
        self.angular_velocity.replace(body.angvel());
    }

    pub(crate) fn update_body(&mut self, body: &mut RigidBody) {
        if let Some(&velocity) = self.velocity.consume_ref_if_changed() {
            body.set_linvel(Vector2::new(velocity.x, velocity.y), true);
        }
        if let Some(&velocity) = self.angular_velocity.consume_ref_if_changed() {
            body.set_angvel(velocity, true);
        }
    }

    pub(crate) fn updated_body_builder(&mut self, builder: RigidBodyBuilder) -> RigidBodyBuilder {
        let velocity = self.velocity.consume_ref();
        builder
            .linvel(Vector2::new(velocity.x, velocity.y))
            .angvel(*self.angular_velocity.consume_ref())
    }
}

impl Default for Dynamics2D {
    fn default() -> Self {
        Self::new()
    }
}
