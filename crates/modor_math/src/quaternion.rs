use crate::{Mat4, Vec3D};
use std::f32::consts::PI;

#[derive(Clone, Copy, Debug)]
pub struct Quaternion {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl Default for Quaternion {
    fn default() -> Self {
        Self::from_components(0., 0., 0., 1.)
    }
}

impl Quaternion {
    #[inline]
    pub const fn from_components(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn from_axis_angle<U>(axis: Vec3D<U>, angle: f32) -> Self {
        let axis = axis.with_magnitude(1.).unwrap_or(Vec3D::ZERO);
        let angle = Self::normalize_angle(angle);
        Self {
            x: axis.x * (angle / 2.).sin(),
            y: axis.y * (angle / 2.).sin(),
            z: axis.z * (angle / 2.).sin(),
            w: (angle / 2.).cos(),
        }
    }

    pub fn axis<U>(self) -> Vec3D<U> {
        if self.w == 1. {
            Vec3D::X
        } else {
            Vec3D::xyz(
                self.x / (1. - self.w.powi(2)).sqrt(),
                self.y / (1. - self.w.powi(2)).sqrt(),
                self.z / (1. - self.w.powi(2)).sqrt(),
            )
        }
    }

    #[inline]
    pub fn angle(self) -> f32 {
        2. * self.w.acos()
    }

    pub fn matrix<U>(self) -> Mat4<U> {
        Mat4::from_array([
            [
                1. - 2. * self.y * self.y - 2. * self.z * self.z,
                2. * self.x * self.y - 2. * self.w * self.z,
                2. * self.x * self.z + 2. * self.w * self.y,
                0.,
            ],
            [
                2. * self.x * self.y + 2. * self.w * self.z,
                1. - 2. * self.x * self.x - 2. * self.z * self.z,
                2. * self.y * self.z - 2. * self.w * self.x,
                0.,
            ],
            [
                2. * self.x * self.z - 2. * self.w * self.y,
                2. * self.y * self.z + 2. * self.w * self.x,
                1. - 2. * self.x * self.x - 2. * self.y * self.y,
                0.,
            ],
            [0., 0., 0., 1.],
        ])
    }

    pub fn with_rotation(self, rotation: Self) -> Self {
        let other = rotation;
        Self {
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
        }
    }

    fn normalize_angle(mut angle: f32) -> f32 {
        while angle > 2. * PI {
            angle -= 2. * PI;
        }
        while angle < 0. {
            angle += 2. * PI;
        }
        angle
    }
}
