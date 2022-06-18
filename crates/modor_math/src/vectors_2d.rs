use crate::Vec3D;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// A vector in a 2D space with `U` as unit of distance.
#[derive(Default, Clone, Copy, Debug)]
pub struct Vec2D<U> {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
    phantom: PhantomData<U>,
}

impl<U> Vec2D<U> {
    /// A vector with all components equal to `0.0`.
    pub const ZERO: Self = Self::xy(0., 0.);

    /// A vector with X component equal to `1.0`.
    pub const X: Self = Self::xy(1., 0.);

    /// A vector with Y component equal to `1.0`.
    pub const Y: Self = Self::xy(0., 1.);

    /// A vector with X component equal to `-1.0`.
    pub const NEG_X: Self = Self::xy(-1., 0.);

    /// A vector with Y component equal to `-1.0`.
    pub const NEG_Y: Self = Self::xy(0., -1.);

    /// Creates a new vector.
    #[inline]
    pub const fn xy(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            phantom: PhantomData,
        }
    }

    /// Converts to a vector with the same coordinates and a different unit of distance `U2`.
    #[inline]
    pub const fn with_unit<U2>(self) -> Vec2D<U2> {
        Vec2D::xy(self.x, self.y)
    }

    /// Converts to a 3D vector with the same x and y coordinates, and a chosen `z` coordinate.
    pub fn with_z(self, z: f32) -> Vec3D<U> {
        Vec3D::xyz(self.x, self.y, z)
    }

    /// Returns the vector with the same direction and but a different `magnitude`.
    ///
    /// If all components of the vector are equal to `0.0`, `None` is returned.
    pub fn with_magnitude(self, magnitude: f32) -> Option<Self> {
        let (x, y) = (self.x, self.y);
        let factor = magnitude / self.magnitude();
        factor.is_finite().then(|| Self::xy(x * factor, y * factor))
    }

    /// Returns the magnitude of the vector.
    pub fn magnitude(self) -> f32 {
        self.x.mul_add(self.x, self.y.powi(2)).sqrt()
    }
}

impl<U> Add<Vec2D<U>> for Vec2D<U> {
    type Output = Vec2D<U>;

    fn add(self, rhs: Vec2D<U>) -> Self::Output {
        Self::xy(self.x + rhs.x, self.y + rhs.y)
    }
}

impl<U> Sub<Vec2D<U>> for Vec2D<U> {
    type Output = Vec2D<U>;

    fn sub(self, rhs: Vec2D<U>) -> Self::Output {
        Self::xy(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<U> Mul<f32> for Vec2D<U> {
    type Output = Vec2D<U>;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::xy(self.x * rhs, self.y * rhs)
    }
}

impl<U> Div<f32> for Vec2D<U> {
    type Output = Vec2D<U>;

    fn div(self, rhs: f32) -> Self::Output {
        Self::xy(self.x / rhs, self.y / rhs)
    }
}

impl<U> AddAssign<Vec2D<U>> for Vec2D<U> {
    fn add_assign(&mut self, rhs: Vec2D<U>) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<U> SubAssign<Vec2D<U>> for Vec2D<U> {
    fn sub_assign(&mut self, rhs: Vec2D<U>) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<U> MulAssign<f32> for Vec2D<U> {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl<U> DivAssign<f32> for Vec2D<U> {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}
