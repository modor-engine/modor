use crate::components::relative_transform::RelativeTransform;
use crate::components::transform::Transform;
use crate::DeltaTime;
use modor_math::{Quat, Vec3};
use std::marker::PhantomData;

/// The dynamic properties of an entity.
///
/// # Modor
///
/// - **Type**: component
/// - **Required components**: [`Transform`](crate::Transform)
/// - **Updated by**: [`PhysicsModule`](crate::PhysicsModule)
/// - **Updated during**: [`UpdatePhysicsAction`](crate::UpdatePhysicsAction)
///
/// # Example
///
/// See [`PhysicsModule`](crate::PhysicsModule).
#[derive(Clone, Debug, Default)]
pub struct DynamicBody {
    /// Linear velocity of the entity in world units per second.
    pub velocity: Vec3,
    /// Linear acceleration of the entity in world units per second squared.
    pub acceleration: Vec3,
    /// Angular velocity of the entity in radians per second.
    pub angular_velocity: Quat,
    /// Angular acceleration of the entity in radians per second squared.
    pub angular_acceleration: Quat,
    phantom: PhantomData<()>,
}

impl DynamicBody {
    /// Creates a new body without movement.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            angular_velocity: Quat::ZERO,
            angular_acceleration: Quat::ZERO,
            phantom: PhantomData,
        }
    }

    /// Returns the transform with a different `velocity`.
    #[must_use]
    #[inline]
    pub const fn with_velocity(mut self, velocity: Vec3) -> Self {
        self.velocity = velocity;
        self
    }

    /// Returns the transform with a different `acceleration`.
    #[must_use]
    #[inline]
    pub const fn with_acceleration(mut self, acceleration: Vec3) -> Self {
        self.acceleration = acceleration;
        self
    }

    /// Returns the transform with a different `angular_velocity`.
    #[must_use]
    #[inline]
    pub const fn with_angular_velocity(mut self, angular_velocity: Quat) -> Self {
        self.angular_velocity = angular_velocity;
        self
    }

    /// Returns the transform with a different `angular_acceleration`.
    #[must_use]
    #[inline]
    pub const fn with_angular_acceleration(mut self, angular_acceleration: Quat) -> Self {
        self.angular_acceleration = angular_acceleration;
        self
    }

    pub(crate) fn update(
        &mut self,
        transform: &mut Transform,
        mut relative: Option<&mut RelativeTransform>,
        delta_time: &DeltaTime,
    ) {
        self.velocity += self.acceleration * delta_time.get().as_secs_f32();
        self.angular_velocity *= self
            .angular_acceleration
            .with_scale(delta_time.get().as_secs_f32());
        let position = relative
            .as_mut()
            .and_then(|r| r.position.as_mut())
            .unwrap_or(&mut transform.position);
        *position += self.velocity * delta_time.get().as_secs_f32();
        let rotation = relative
            .as_mut()
            .and_then(|r| r.rotation.as_mut())
            .unwrap_or(&mut transform.rotation);
        *rotation *= self
            .angular_velocity
            .with_scale(delta_time.get().as_secs_f32());
    }
}
