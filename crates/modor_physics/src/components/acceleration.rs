/// A component storing the acceleration of an entity.
///
/// This component is effective only if the entity also has a [`Velocity`](crate::Velocity).<br>
/// [`PhysicsModule`](crate::PhysicsModule) automatically updates the velocity from the
/// acceleration.
///
/// The acceleration is measured in distance units per second squared.<br>
/// A distance unit of 1 on an axis corresponds to the size along this axis of the first
/// parent having a position.
///
/// # Examples
///
/// See [`PhysicsModule`](crate::PhysicsModule).
#[derive(Clone, Debug)]
pub struct Acceleration {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
    /// The Z-coordinate.
    pub z: f32,
}

impl Acceleration {
    /// Creates a 3D acceleration.
    pub fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Creates a 2D acceleration.
    ///
    /// Z-coordinate is set to zero.
    pub fn xy(x: f32, y: f32) -> Self {
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
}

#[cfg(test)]
mod acceleration_tests {
    use crate::Acceleration;
    use approx::assert_abs_diff_eq;

    #[test]
    fn create_2d() {
        let acceleration = Acceleration::xy(1., 2.);
        assert_abs_diff_eq!(acceleration.x, 1.);
        assert_abs_diff_eq!(acceleration.y, 2.);
        assert_abs_diff_eq!(acceleration.z, 0.);
    }

    #[test]
    fn create_3d() {
        let acceleration = Acceleration::xyz(1., 2., 3.);
        assert_abs_diff_eq!(acceleration.x, 1.);
        assert_abs_diff_eq!(acceleration.y, 2.);
        assert_abs_diff_eq!(acceleration.z, 3.);
    }

    #[test]
    fn use_() {
        let mut acceleration = Acceleration::xyz(1., 2., 3.);
        assert_abs_diff_eq!(acceleration.magnitude(), 14.0_f32.sqrt());
        acceleration.set_magnitude(14.0_f32.sqrt() * 2.);
        assert_abs_diff_eq!(acceleration.x, 2.);
        assert_abs_diff_eq!(acceleration.y, 4.);
        assert_abs_diff_eq!(acceleration.z, 6.);
        acceleration.set_magnitude(0.);
        acceleration.set_magnitude(1.);
        assert_abs_diff_eq!(acceleration.x, 0.);
        assert_abs_diff_eq!(acceleration.y, 0.);
        assert_abs_diff_eq!(acceleration.z, 0.);
    }
}
