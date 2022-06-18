use crate::{Size3D, Vec2D};
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// A size in a 2D space with `U` as unit of distance.
#[derive(Default, Clone, Copy, Debug)]
pub struct Size2D<U> {
    /// The width.
    pub x: f32,
    /// The height.
    pub y: f32,
    phantom: PhantomData<U>,
}

impl<U> Size2D<U> {
    /// A size with all components equal to `0.0`.
    pub const ZERO: Self = Self::xy(0., 0.);

    /// A size with all components equal to `1.0`.
    pub const ONE: Self = Self::xy(1., 1.);

    /// Creates a new size.
    #[inline]
    pub const fn xy(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            phantom: PhantomData,
        }
    }

    /// Converts to a point with the same coordinates and a different unit of distance `U2`.
    #[inline]
    pub const fn with_unit<U2>(self) -> Size2D<U2> {
        Size2D::xy(self.x, self.y)
    }

    /// Converts to a 3D size with the same x and y coordinates, and a chosen `z` coordinate.
    #[inline]
    pub fn with_z(self, z: f32) -> Size3D<U> {
        Size3D::xyz(self.x, self.y, z)
    }

    /// Converts to a vector with same coordinates.
    #[inline]
    pub fn to_vec(self) -> Vec2D<U> {
        Vec2D::xy(self.x, self.y)
    }
}

impl<U> Add<Vec2D<U>> for Size2D<U> {
    type Output = Size2D<U>;

    fn add(self, rhs: Vec2D<U>) -> Self::Output {
        Self::xy(self.x + rhs.x, self.y + rhs.y)
    }
}

impl<U> Sub<Vec2D<U>> for Size2D<U> {
    type Output = Size2D<U>;

    fn sub(self, rhs: Vec2D<U>) -> Self::Output {
        Self::xy(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<U> Mul<f32> for Size2D<U> {
    type Output = Size2D<U>;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::xy(self.x * rhs, self.y * rhs)
    }
}

impl<U> Div<f32> for Size2D<U> {
    type Output = Size2D<U>;

    fn div(self, rhs: f32) -> Self::Output {
        Self::xy(self.x / rhs, self.y / rhs)
    }
}

impl<U> AddAssign<Vec2D<U>> for Size2D<U> {
    fn add_assign(&mut self, rhs: Vec2D<U>) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<U> SubAssign<Vec2D<U>> for Size2D<U> {
    fn sub_assign(&mut self, rhs: Vec2D<U>) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<U> MulAssign<f32> for Size2D<U> {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl<U> DivAssign<f32> for Size2D<U> {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}
