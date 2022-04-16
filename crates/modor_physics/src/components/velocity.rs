use crate::Acceleration;
use std::time::Duration;

/// A component storing the velocity of an entity.
///
/// This component is effective only if the entity also has a [`Position`](crate::Position).<br>
/// [`PhysicsModule`](crate::PhysicsModule) automatically updates the position from the
/// velocity.
///
/// The velocity is measured in distance units per second.<br>
/// A distance unit of 1 on an axis corresponds to the size along this axis of the first
/// parent having a position.
///
/// # Examples
///
/// See [`PhysicsModule`](crate::PhysicsModule).
#[derive(Clone, Debug)]
pub struct Velocity {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
    /// The Z-coordinate.
    pub z: f32,
}

impl Velocity {
    /// Creates a 3D velocity.
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Creates a 2D velocity.
    ///
    /// Z-coordinate is set to zero.
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 0.)
    }

    /// Returns the magnitude.
    pub fn magnitude(&self) -> f32 {
        self.x
            .mul_add(self.x, self.y.mul_add(self.y, self.z.powi(2)))
            .sqrt()
    }

    /// Set the magnitude.
    ///
    /// If the current magnitude of the acceleration is zero, the magnitude is unchanged.
    pub fn set_magnitude(&mut self, magnitude: f32) -> &mut Self {
        let factor = magnitude / self.magnitude();
        if factor.is_finite() {
            self.x *= factor;
            self.y *= factor;
            self.z *= factor;
        }
        self
    }

    pub(crate) fn update(&mut self, acceleration: &Acceleration, delta_time: Duration) {
        self.x += acceleration.x * delta_time.as_secs_f32();
        self.y += acceleration.y * delta_time.as_secs_f32();
        self.z += acceleration.z * delta_time.as_secs_f32();
    }
}

#[cfg(test)]
mod velocity_tests {
    use crate::Velocity;
    use approx::assert_abs_diff_eq;

    #[test]
    fn create_2d() {
        let velocity = Velocity::xy(1., 2.);
        assert_abs_diff_eq!(velocity.x, 1.);
        assert_abs_diff_eq!(velocity.y, 2.);
        assert_abs_diff_eq!(velocity.z, 0.);
    }

    #[test]
    fn create_3d() {
        let velocity = Velocity::xyz(1., 2., 3.);
        assert_abs_diff_eq!(velocity.x, 1.);
        assert_abs_diff_eq!(velocity.y, 2.);
        assert_abs_diff_eq!(velocity.z, 3.);
    }

    #[test]
    fn use_() {
        let mut velocity = Velocity::xyz(1., 2., 3.);
        assert_abs_diff_eq!(velocity.magnitude(), 14.0_f32.sqrt());
        velocity.set_magnitude(14.0_f32.sqrt() * 2.);
        assert_abs_diff_eq!(velocity.x, 2.);
        assert_abs_diff_eq!(velocity.y, 4.);
        assert_abs_diff_eq!(velocity.z, 6.);
        velocity.set_magnitude(0.);
        velocity.set_magnitude(1.);
        assert_abs_diff_eq!(velocity.x, 0.);
        assert_abs_diff_eq!(velocity.y, 0.);
        assert_abs_diff_eq!(velocity.z, 0.);
    }
}
