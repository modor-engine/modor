/// The acceleration of an entity.
///
/// The acceleration is measured in distance units per second squared.<br>
/// A distance unit of 1 on an axis corresponds to the size along this axis of the first
/// parent having a position.
///
/// # Modor
///
/// - **Type**: component
/// - **Required components**: [`Velocity`](crate::Velocity)
/// - **Default if missing**: `Acceleration::xyz(0., 0., 0.)`
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
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Creates a 2D acceleration.
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
}
