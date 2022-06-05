use std::ops::{Add, AddAssign, Sub, SubAssign};

/// A trait for defining a vector in a 2D space.
pub trait Vector2D: Copy + Add + Sub + AddAssign + SubAssign {
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
    ///
    /// If all components of the initial vector are equal to zero, the returned vector is the same.
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

// impl<V> AddAssign for V
// where
//     V: Vector2D,
// {
//     fn add_assign(&mut self, rhs: Self) {
//         *self = Self::create(self.x + rhs.x, self.y + rhs.y);
//     }
// }

/// A trait for defining a vector in a 3D space.
pub trait Vector3D: Copy + Add + Sub + AddAssign + SubAssign {
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
    ///
    /// If all components of the initial vector are equal to zero, the returned vector is the same.
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
