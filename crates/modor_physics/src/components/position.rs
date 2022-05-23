use crate::{Scale, Velocity};
use std::time::Duration;

/// The position of an entity.
///
/// The position is relative to the first parent entity also having a position.<br>
/// This is an absolute position in case the entity does not have any parent with a position.
///
/// # Modor
///
/// - **Type**: component
/// - **Required components**: [`Scale`](crate::Scale)
/// - **Updated by**: [`PhysicsModule`](crate::PhysicsModule)
/// - **Updated using**: [`Velocity`](crate::Velocity), [`Position`](crate::Position)
///     of parent entity, [`Scale`](crate::Scale) of parent entity, [`DeltaTime`](crate::DeltaTime)
/// - **Updated during**: [`UpdatePhysicsAction`](crate::UpdatePhysicsAction)
///
/// # Examples
///
/// See [`PhysicsModule`](crate::PhysicsModule).
#[derive(Clone, Debug)]
pub struct Position {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
    /// The Z-coordinate.
    pub z: f32,
    abs: AbsolutePosition,
}

impl Position {
    /// Creates a 3D position.
    ///
    /// Absolute position is initialized with the same coordinates.
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
            abs: AbsolutePosition { x, y, z },
        }
    }

    /// Creates a 2D position.
    ///
    /// Z-coordinate is set to zero.
    ///
    /// Absolute position is initialized with the same coordinates.
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 0.)
    }

    /// Returns the absolute position.
    pub fn abs(&self) -> &AbsolutePosition {
        &self.abs
    }

    /// Returns the distance between the position and `other_position`.
    ///
    /// Absolute position is used to calculate the distance.
    pub fn distance(&self, other_position: &Self) -> f32 {
        let x_diff = self.abs.x - other_position.abs.x;
        let y_diff = self.abs.y - other_position.abs.y;
        let z_diff = self.abs.z - other_position.abs.z;
        x_diff
            .mul_add(x_diff, y_diff.mul_add(y_diff, z_diff.powi(2)))
            .sqrt()
    }

    pub(crate) fn update(&mut self, velocity: &Velocity, delta_time: Duration) {
        self.x += velocity.x * delta_time.as_secs_f32();
        self.y += velocity.y * delta_time.as_secs_f32();
        self.z += velocity.z * delta_time.as_secs_f32();
    }

    pub(crate) fn update_abs(&mut self, parent_position: &Self, parent_scale: &Scale) {
        self.abs.x = self.x.mul_add(parent_scale.abs().x, parent_position.abs.x);
        self.abs.y = self.y.mul_add(parent_scale.abs().y, parent_position.abs.y);
        self.abs.z = self.z.mul_add(parent_scale.abs().z, parent_position.abs.z);
    }
}

// TODO: split in RelativePosition and Position

/// An absolute position corresponding to a relative [`Position`](crate::Position).
#[derive(Clone, Copy, Debug)]
pub struct AbsolutePosition {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
    /// The Z-coordinate.
    pub z: f32,
}
