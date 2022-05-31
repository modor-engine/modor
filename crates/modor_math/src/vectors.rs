/// A trait for defining a vector in a 2D space.
pub trait Vector2D: Copy {
    /// Creates a new vector.
    fn create(x: f32, y: f32) -> Self;

    /// Returns components of the point.
    fn components(self) -> (f32, f32);

    /// Returns the magnitude of the vector.
    fn magnitude(self) -> f32 {
        let (x, y) = self.components();
        x.mul_add(x, y.powi(2)).sqrt()
    }

    /// Returns the vector with the same direction but another `magnitude`.
    #[must_use]
    fn with_magnitude(self, magnitude: f32) -> Self {
        let (x, y) = self.components();
        let factor = magnitude / self.magnitude();
        if factor.is_finite() {
            Self::create(x * factor, y * factor)
        } else {
            Self::create(0., 0.)
        }
    }
}

/// A trait for defining a vector in a 3D space.
pub trait Vector3D: Copy {
    /// Creates a new vector.
    fn create(x: f32, y: f32, z: f32) -> Self;

    /// Returns components of the point.
    fn components(self) -> (f32, f32, f32);

    /// Returns the magnitude of the vector.
    fn magnitude(self) -> f32 {
        let (x, y, z) = self.components();
        x.mul_add(x, y.mul_add(y, z.powi(2))).sqrt()
    }

    /// Returns the vector with the same direction but another `magnitude`.
    #[must_use]
    fn with_magnitude(self, magnitude: f32) -> Self {
        let (x, y, z) = self.components();
        let factor = magnitude / self.magnitude();
        if factor.is_finite() {
            Self::create(x * factor, y * factor, z * factor)
        } else {
            Self::create(0., 0., 0.)
        }
    }
}
