use crate::components::pipeline::{BodyHandleReset, Pipeline2D, UnsynchronizedHandleDeletion};
use crate::Transform2D;
use modor::{Entity, Filter, Not, SingleMut, With};
use modor_math::Vec2;
use rapier2d::dynamics::{RigidBodyBuilder, RigidBodyHandle, RigidBodyType};
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
/// - [`Transform2D`] component is in the same entity
///
/// # Related components
///
/// - [`Transform2D`]
/// - [`Collider2D`](crate::Collider2D)
///
/// # Example
///
/// ```rust
/// # use modor::*;
/// # use modor_math::*;
/// # use modor_physics::*;
/// #
/// App::new()
///     .with_entity(modor_physics::module())
///     .with_entity(object());
///
/// fn object() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .component(Transform2D::new())
///         .component(Dynamics2D::new())
///         .with(|d| d.velocity = Vec2::new(0.5, 0.2))
/// }
/// ```
#[derive(Component, Debug)]
pub struct Dynamics2D {
    /// Linear velocity of the entity in world units per second.
    ///
    /// Default value is `Vec2::ZERO`.
    pub velocity: Vec2,
    /// Angular velocity of the entity in radians per second.
    ///
    /// Default value is `0.0`.
    pub angular_velocity: f32,
    // TODO: implement the following fields:
    // pub force: Vec2,
    // pub torque: Vec2,
    // pub density: f32,
    // pub damping: f32,
    // pub dominance: i8,
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

    #[run_as(action(BodyHandleReset))]
    fn reset_handle_if_transform_removed(&mut self, _filter: Filter<Not<With<Transform2D>>>) {
        self.handle = None;
    }

    #[run_after(action(UnsynchronizedHandleDeletion))]
    fn update_pipeline(
        &mut self,
        transform: &mut Transform2D,
        entity: Entity<'_>,
        mut pipeline: SingleMut<'_, '_, Pipeline2D>,
    ) {
        let pipeline = pipeline.get_mut();
        if let Some(body) = self.handle.and_then(|handle| pipeline.body_mut(handle)) {
            body.set_translation(vector![transform.position.x, transform.position.y], true);
            body.set_rotation(Rotation::new(transform.rotation), true);
            body.set_linvel(vector![self.velocity.x, self.velocity.y], true);
            body.set_angvel(self.angular_velocity, true);
            body.user_data = entity.id() as u128;
        } else {
            let builder = self.body_builder(entity.id(), transform);
            self.handle = Some(pipeline.create_body(builder));
        }
    }

    fn body_builder(&self, entity_id: usize, transform: &mut Transform2D) -> RigidBodyBuilder {
        RigidBodyBuilder::new(RigidBodyType::Dynamic)
            .can_sleep(false)
            .translation(vector![transform.position.x, transform.position.y])
            .rotation(transform.rotation)
            .linvel(vector![self.velocity.x, self.velocity.y])
            .angvel(self.angular_velocity)
            .user_data(entity_id as u128)
    }
}

impl Default for Dynamics2D {
    fn default() -> Self {
        Self::new()
    }
}
