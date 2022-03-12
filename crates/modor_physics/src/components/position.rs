use crate::{Scale, Velocity};
use std::time::Duration;

/// A component storing the position of an entity.
///
/// The position is relative to the first parent entity also having a position.<br>
/// This is an absolute position in case the entity does not have any parent with a position.
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
    pub(crate) abs: AbsolutePosition,
}

impl Position {
    /// Creates a 3D position.
    ///
    /// Absolute position is initialized with the same coordinates.
    pub fn xyz(x: f32, y: f32, z: f32) -> Self {
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
    pub fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 0.)
    }

    /// Returns the absolute position.
    ///
    /// The absolute position is automatically calculated by the
    /// [`PhysicsModule`](crate::PhysicsModule).<br>
    /// If your system needs to access the absolute position, then it can depend on
    /// [`PhysicsUpdateAction`](crate::PhysicsUpdateAction) to make sure to use an up-to-date
    /// position.
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

    pub(crate) fn update_abs(&mut self, parent_position: &Self, parent_scale: Option<&Scale>) {
        self.abs.x = self.x.mul_add(
            parent_scale.map_or(1., |s| s.abs().x),
            parent_position.abs.x,
        );
        self.abs.y = self.y.mul_add(
            parent_scale.map_or(1., |s| s.abs().y),
            parent_position.abs.y,
        );
        self.abs.z = self.z.mul_add(
            parent_scale.map_or(1., |s| s.abs().z),
            parent_position.abs.z,
        );
    }
}

/// An absolute position corresponding to a relative [`Position`](crate::Position).
#[derive(Clone, Debug)]
pub struct AbsolutePosition {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
    /// The Z-coordinate.
    pub z: f32,
}

#[cfg(test)]
mod position_tests {
    use crate::Position;
    use approx::assert_abs_diff_eq;

    #[test]
    fn create_2d() {
        let position = Position::xy(1., 2.);
        assert_abs_diff_eq!(position.x, 1.);
        assert_abs_diff_eq!(position.y, 2.);
        assert_abs_diff_eq!(position.z, 0.);
        assert_abs_diff_eq!(position.abs().x, 1.);
        assert_abs_diff_eq!(position.abs().y, 2.);
        assert_abs_diff_eq!(position.abs().z, 0.);
    }

    #[test]
    fn create_3d() {
        let position = Position::xyz(1., 2., 3.);
        assert_abs_diff_eq!(position.x, 1.);
        assert_abs_diff_eq!(position.y, 2.);
        assert_abs_diff_eq!(position.z, 3.);
        assert_abs_diff_eq!(position.abs().x, 1.);
        assert_abs_diff_eq!(position.abs().y, 2.);
        assert_abs_diff_eq!(position.abs().z, 3.);
    }

    #[test]
    fn use_() {
        let mut position1 = Position::xyz(0., 0., 0.);
        position1.abs.x = 1.;
        position1.abs.y = 2.;
        position1.abs.z = 3.;
        let mut position2 = Position::xyz(0., 0., 0.);
        position2.abs.x = 4.;
        position2.abs.y = 6.;
        position2.abs.z = 8.;
        assert_abs_diff_eq!(position1.distance(&position2), 50.0_f32.sqrt());
        assert_abs_diff_eq!(position2.distance(&position1), 50.0_f32.sqrt());
    }
}