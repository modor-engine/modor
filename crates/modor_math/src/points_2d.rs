use crate::{Point3D, Vec2D};
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Sub, SubAssign};

// TODO: Remove PointXD and SizeXD and replace by VecXD
// TODO: use units like
//  - Vec3D<Position>
//  - Vec3D<Velocity>
//  - Vec3D<Acceleration>
//  - Vec3D<RelativePosition>
//  - Vec3D<RelativeVelocity>
//  - Vec3D<RelativeAcceleration>
//  - Vec2D<WindowPosition>
//  - Vec2D<InputDelta>
//  - ...
// TODO: put default value for U=() (useful for quaternions)

/// A point in a 2D space with `U` as unit of distance.
#[derive(Default, Clone, Copy, Debug)]
pub struct Point2D<U> {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
    phantom: PhantomData<U>,
}

impl<U> Point2D<U> {
    /// Creates a new point.
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
    pub const fn with_unit<U2>(self) -> Point2D<U2> {
        Point2D::xy(self.x, self.y)
    }

    /// Converts to a 3D point with the same x and y coordinates, and a chosen `z` coordinate.
    #[inline]
    pub const fn with_z(self, z: f32) -> Point3D<U> {
        Point3D::xyz(self.x, self.y, z)
    }

    /// Returns the Euclidean distance with `other`.
    pub fn distance(self, other: Self) -> f32 {
        let x_diff = self.x - other.x;
        let y_diff = self.y - other.y;
        x_diff.mul_add(x_diff, y_diff.powi(2)).sqrt()
    }
}

impl<U> Add<Vec2D<U>> for Point2D<U> {
    type Output = Point2D<U>;

    fn add(self, rhs: Vec2D<U>) -> Self::Output {
        Self::xy(self.x + rhs.x, self.y + rhs.y)
    }
}

impl<U> Sub<Vec2D<U>> for Point2D<U> {
    type Output = Point2D<U>;

    fn sub(self, rhs: Vec2D<U>) -> Self::Output {
        Self::xy(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<U> AddAssign<Vec2D<U>> for Point2D<U> {
    fn add_assign(&mut self, rhs: Vec2D<U>) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<U> SubAssign<Vec2D<U>> for Point2D<U> {
    fn sub_assign(&mut self, rhs: Vec2D<U>) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}
