use crate::Vec3D;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// A size in a 3D space with `U` as unit of distance.
#[derive(Default, Clone, Copy, Debug)]
pub struct Size3D<U> {
    /// The width.
    pub x: f32,
    /// The height.
    pub y: f32,
    /// The length.
    pub z: f32,
    phantom: PhantomData<U>,
}

impl<U> Size3D<U> {
    /// A size with all components equal to `0.0`.
    pub const ZERO: Self = Self::xyz(0., 0., 0.);

    /// A size with all components equal to `1.0`.
    pub const ONE: Self = Self::xyz(1., 1., 1.);

    /// Creates a new size.
    #[inline]
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
            phantom: PhantomData,
        }
    }

    /// TODO: should we keep xy ? + should we rename xyz into new ? (for modor_math only)
    /// Creates a new size from 2D coordinates.
    ///
    /// Z-coordinate is initialized to `1.0`.
    #[inline]
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 1.) // TODO: replace z by 0.0 + update doc
    }

    /// Converts to a size with the same coordinates and a different unit of distance `U2`.
    #[inline]
    pub const fn with_unit<U2>(self) -> Size3D<U2> {
        Size3D::xyz(self.x, self.y, self.z)
    }

    /// Converts to a vector with same coordinates.
    #[inline]
    pub fn to_vec(self) -> Vec3D<U> {
        Vec3D::xyz(self.x, self.y, self.z)
    }
}

impl<U> Add<Vec3D<U>> for Size3D<U> {
    type Output = Size3D<U>;

    fn add(self, rhs: Vec3D<U>) -> Self::Output {
        Self::xyz(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl<U> Sub<Vec3D<U>> for Size3D<U> {
    type Output = Size3D<U>;

    fn sub(self, rhs: Vec3D<U>) -> Self::Output {
        Self::xyz(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl<U> Mul<f32> for Size3D<U> {
    type Output = Size3D<U>;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::xyz(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl<U> Div<f32> for Size3D<U> {
    type Output = Size3D<U>;

    fn div(self, rhs: f32) -> Self::Output {
        Self::xyz(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl<U> AddAssign<Vec3D<U>> for Size3D<U> {
    fn add_assign(&mut self, rhs: Vec3D<U>) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl<U> SubAssign<Vec3D<U>> for Size3D<U> {
    fn sub_assign(&mut self, rhs: Vec3D<U>) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl<U> MulAssign<f32> for Size3D<U> {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl<U> DivAssign<f32> for Size3D<U> {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}
