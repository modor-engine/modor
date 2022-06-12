use crate::Vector3D;
use std::f32::consts::PI;

// TODO: document + test

pub trait Quaternion: Copy {
    type Axis: Vector3D;

    fn create(x: f32, y: f32, z: f32, w: f32) -> Self;

    fn components(self) -> (f32, f32, f32, f32);

    fn create_from_axis_angle(axis: Self::Axis, mut angle: f32) -> Self {
        while angle > 2. * PI {
            angle -= 2. * PI;
        }
        while angle < 0. {
            angle += 2. * PI;
        }
        let (axis_x, axis_y, axis_z) = axis.with_magnitude(1.).components();
        Self::create(
            axis_x * (angle / 2.).sin(),
            axis_y * (angle / 2.).sin(),
            axis_z * (angle / 2.).sin(),
            (angle / 2.).cos(),
        )
    }

    fn axis(self) -> Self::Axis {
        let (x, y, z, w) = self.components();
        if w == 1. {
            Self::Axis::create(1., 0., 0.)
        } else {
            Self::Axis::create(
                x / (1. - w * w).sqrt(),
                y / (1. - w * w).sqrt(),
                z / (1. - w * w).sqrt(),
            )
        }
    }

    fn angle(self) -> f32 {
        let (_, _, _, w) = self.components();
        2. * w.acos()
    }

    fn matrix(self) -> [[f32; 4]; 4] {
        let (x, y, z, w) = self.components();
        [
            [
                1. - 2. * y * y - 2. * z * z,
                2. * x * y - 2. * w * z,
                2. * x * z + 2. * w * y,
                0.,
            ],
            [
                2. * x * y + 2. * w * z,
                1. - 2. * x * x - 2. * z * z,
                2. * y * z - 2. * w * x,
                0.,
            ],
            [
                2. * x * z - 2. * w * y,
                2. * y * z + 2. * w * x,
                1. - 2. * x * x - 2. * y * y,
                0.,
            ],
            [0., 0., 0., 1.],
        ]
    }

    fn with_rotation<Q>(self, quaternion: Q) -> Self
    where
        Q: Quaternion,
    {
        let (d, a, b, c) = quaternion.components();
        let (h, e, f, g) = self.components();
        Self::create(
            b * e + a * f + c * h - d * g,
            a * g - b * h + c * e + d * f,
            a * h + b * g - c * f + d * e,
            a * e - b * f - c * g - d * h,
        )
    }
}
