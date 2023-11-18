use crate::components::pipeline::{BodyHandleReset, Pipeline2D, UnsynchronizedHandleDeletion};
use crate::Transform2D;
use modor::{Entity, Filter, Not, SingleMut, With};
use modor_math::Vec2;
use rapier2d::dynamics::{
    MassProperties, RigidBody, RigidBodyBuilder, RigidBodyHandle, RigidBodyType,
};
use rapier2d::math::Rotation;
use rapier2d::na::{point, vector};
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
    /// Force applied on the entity.
    ///
    /// Will not have any effect if mass is zero.
    ///
    /// The acceleration of the entity corresponds to the force of the entity divided by its mass.
    ///
    /// Default value is [`Vec2::ZERO`].
    pub force: Vec2,
    /// Torque applied on the entity.
    ///
    /// Will not have any effect if angular inertia is zero.
    ///
    /// Default value is `0.`.
    pub torque: f32,
    /// Mass of the entity.
    ///
    /// A mass of zero is considered as infinite. In this case, force will not have any effect
    /// (even in case of collisions).
    ///
    /// Default value is `0.`.
    pub mass: f32,
    /// Angular inertia of the entity.
    ///
    /// An angular inertia of zero is considered as infinite. In this case, torque will not have
    /// any effect (even in case of collisions).
    ///
    /// Default value is `0.`.
    pub angular_inertia: f32,
    /// Linear damping of the entity.
    ///
    /// This coefficient is used to automatically slow down the translation of the entity.
    ///
    /// Default value is `0.`.
    pub damping: f32,
    /// Angular damping of the entity.
    ///
    /// This coefficient is used to automatically slow down the rotation of the entity.
    ///
    /// Default value is `0.`.
    pub angular_damping: f32,
    /// Dominance of the entity.
    ///
    /// In case of collision between two entities, if both entities have a different dominance
    /// group, then collision forces will only be applied on the entity with the smallest dominance.
    ///
    /// Default value is `0`.
    pub dominance: i8,
    /// Whether Continuous Collision Detection is enabled for the entity.
    ///
    /// This option is used to detect a collision even if the entity moves too fast.
    /// CCD is performed using motion-clamping, which means each fast-moving entity with CCD enabled
    /// will be stopped at the moment of their first contact. Both angular and translational motions
    /// are taken into account.
    ///
    /// Note that CCD require additional computation, so it is recommended to enable it only for
    /// entities that are expected to move fast.
    ///
    /// Default value is `false`.
    pub is_ccd_enabled: bool,
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
            force: Vec2::ZERO,
            torque: 0.,
            mass: 0.,
            angular_inertia: 0.,
            damping: 0.,
            angular_damping: 0.,
            dominance: 0,
            is_ccd_enabled: false,
            handle: None,
        }
    }

    #[run_as(action(BodyHandleReset))]
    fn reset_handle_if_transform_removed(&mut self, _filter: Filter<Not<With<Transform2D>>>) {
        self.handle = None;
    }

    #[run_as(action(BodyUpdate))]
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
            body.reset_forces(true);
            body.add_force(vector![self.force.x, self.force.y], true);
            body.reset_torques(true);
            body.add_torque(self.torque, true);
            let mass = MassProperties::new(point![0., 0.], self.mass, self.angular_inertia);
            body.set_additional_mass_properties(mass, true);
            body.set_linear_damping(self.damping);
            body.set_angular_damping(self.angular_damping);
            body.set_dominance_group(self.dominance);
            body.enable_ccd(self.is_ccd_enabled);
            body.user_data = entity.id() as u128;
        } else {
            let body = self.create_body(entity.id(), transform);
            self.handle = Some(pipeline.create_body(body));
        }
    }

    #[run_after(component(Pipeline2D))]
    fn update(&mut self, mut pipeline: SingleMut<'_, '_, Pipeline2D>) {
        let pipeline = pipeline.get_mut();
        if let Some(body) = self.handle.and_then(|handle| pipeline.body_mut(handle)) {
            self.velocity.x = body.linvel().x;
            self.velocity.y = body.linvel().y;
            self.angular_velocity = body.angvel();
            self.force.x = body.user_force().x;
            self.force.y = body.user_force().y;
            self.torque = body.user_torque();
        }
    }

    fn create_body(&self, entity_id: usize, transform: &Transform2D) -> RigidBody {
        let mass = MassProperties::new(point![0., 0.], self.mass, self.angular_inertia);
        let mut body = RigidBodyBuilder::new(RigidBodyType::Dynamic)
            .can_sleep(false)
            .translation(vector![transform.position.x, transform.position.y])
            .rotation(transform.rotation)
            .linvel(vector![self.velocity.x, self.velocity.y])
            .angvel(self.angular_velocity)
            .additional_mass_properties(mass)
            .linear_damping(self.damping)
            .angular_damping(self.angular_damping)
            .dominance_group(self.dominance)
            .ccd_enabled(self.is_ccd_enabled)
            .user_data(entity_id as u128)
            .build();
        body.add_force(vector![self.force.x, self.force.y], true);
        body.add_torque(self.torque, true);
        body
    }
}

impl Default for Dynamics2D {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Action)]
pub(crate) struct BodyUpdate(UnsynchronizedHandleDeletion);
