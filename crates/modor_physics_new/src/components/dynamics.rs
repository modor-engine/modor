use crate::components::pipeline::{HandleRemoval, Pipeline2D};
use crate::Transform2D;
use modor::{Entity, Filter, Not, SingleMut, With};
use modor_math::Vec2;
use rapier2d::dynamics::{RigidBody, RigidBodyBuilder, RigidBodyHandle, RigidBodyType};
use rapier2d::math::Rotation;
use rapier2d::na::vector;
use rapier2d::prelude::nalgebra;

/// The dynamics properties of a 2D entity.
///
/// This component has an effect only if the entity has also a component of type
/// [`Transform2D`](Transform2D).
///
/// # Requirements
///
/// The component is effective only if:
/// - physics [`module`](crate::module()) is initialized
///
/// # Related components
///
/// - [`Transform2D`]
///
/// # Example
///
/// See [`PhysicsModule`](crate::PhysicsModule).
#[derive(Component, Debug, Clone)]
pub struct Dynamics2D {
    /// Linear velocity of the entity in world units per second.
    ///
    /// Default value is `Vec2::ZERO`.
    pub velocity: Vec2,
    /// Angular velocity of the entity in radians per second.
    ///
    /// Default value is `0.0`.
    pub angular_velocity: f32,
    pub(crate) handle: Option<RigidBodyHandle>,
}

#[systems]
impl Dynamics2D {
    /// Creates a new body.
    #[inline]
    pub const fn new() -> Self {
        Self {
            velocity: Vec2::ZERO,
            angular_velocity: 0.,
            handle: None,
        }
    }

    #[run_as(action(HandleRemoval))]
    fn reset_handle_if_transform_removed(&mut self, _filter: Filter<Not<With<Transform2D>>>) {
        self.handle = None;
    }

    #[run_after(component(Pipeline2D))]
    fn update(
        &mut self,
        transform: &mut Transform2D,
        entity: Entity<'_>,
        mut pipeline: SingleMut<'_, '_, Pipeline2D>,
    ) {
        let pipeline = pipeline.get_mut();
        if let Some(body) = self.handle.and_then(|handle| pipeline.body_mut(handle)) {
            Self::update_position(transform, body);
            Self::update_rotation(transform, body);
            self.update_velocity(body);
            self.update_angular_velocity(body);
            body.user_data = entity.id() as u128;
        } else {
            let builder = self.body_builder(entity.id(), transform);
            self.handle = Some(pipeline.create_body(builder));
        }
    }

    fn body_builder(&mut self, entity_id: usize, transform: &mut Transform2D) -> RigidBodyBuilder {
        RigidBodyBuilder::new(RigidBodyType::Dynamic)
            .can_sleep(false)
            .translation(vector![transform.position.x, transform.position.y])
            .rotation(transform.rotation)
            .linvel(vector![self.velocity.x, self.velocity.y])
            .angvel(self.angular_velocity)
            .user_data(entity_id as u128)
    }

    fn update_position(transform: &mut Transform2D, body: &mut RigidBody) {
        if transform.position == transform.old_position {
            transform.position.x = body.translation().x;
            transform.position.y = body.translation().y;
        } else {
            body.set_translation(vector![transform.position.x, transform.position.y], true);
        }
    }

    #[allow(clippy::float_cmp)]
    fn update_rotation(transform: &mut Transform2D, body: &mut RigidBody) {
        if transform.rotation == transform.old_rotation {
            transform.rotation = body.rotation().angle();
        } else {
            body.set_rotation(Rotation::new(transform.rotation), true);
        }
    }

    fn update_velocity(&mut self, body: &mut RigidBody) {
        body.set_linvel(vector![self.velocity.x, self.velocity.y], true);
    }

    #[allow(clippy::float_cmp)]
    fn update_angular_velocity(&mut self, body: &mut RigidBody) {
        body.set_angvel(self.angular_velocity, true);
    }
}

impl Default for Dynamics2D {
    fn default() -> Self {
        Self::new()
    }
}
