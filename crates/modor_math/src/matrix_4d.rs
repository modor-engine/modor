use crate::{Point3D, Vec3D};
use std::marker::PhantomData;
use std::ops::Mul;

pub struct Mat4<U> {
    elements: [[f32; 4]; 4],
    phantom: PhantomData<U>,
}

impl<U> Mat4<U> {
    #[inline]
    pub fn from_array(elements: [[f32; 4]; 4]) -> Self {
        Self {
            elements,
            phantom: PhantomData,
        }
    }

    pub fn from_position_scale(position: Vec3D<U>, size: Vec3D<U>) -> Self {
        Self::from_array([
            [size.x, 0., 0., 0.],
            [0., size.y, 0., 0.],
            [0., 0., size.z, 0.],
            [position.x, position.y, position.z, 1.],
        ])
    }

    pub fn to_array(&self) -> [[f32; 4]; 4] {
        self.elements
    }

    fn multiply_matrix_part(part: &[f32; 4], other_matrix: &[[f32; 4]; 4], j: usize) -> f32 {
        (0..4)
            .map(|k| part[k] * other_matrix[k][j])
            .reduce(|a, b| a + b)
            .expect("internal error: wrong matrix size")
    }
}

impl<U> Mul<Point3D<U>> for Mat4<U> {
    type Output = Point3D<U>;

    fn mul(self, rhs: Point3D<U>) -> Self::Output {
        let point = [rhs.x, rhs.y, rhs.z, 1.];
        Point3D::xyz(
            Self::multiply_matrix_part(&point, &self.elements, 0),
            Self::multiply_matrix_part(&point, &self.elements, 1),
            Self::multiply_matrix_part(&point, &self.elements, 2),
        )
    }
}

impl<U> Mul<Mat4<U>> for Mat4<U> {
    type Output = Mat4<U>;

    fn mul(self, rhs: Mat4<U>) -> Self::Output {
        Mat4::from_array([
            [
                Self::multiply_matrix_part(&self.elements[0], &rhs.elements, 0),
                Self::multiply_matrix_part(&self.elements[0], &rhs.elements, 1),
                Self::multiply_matrix_part(&self.elements[0], &rhs.elements, 2),
                Self::multiply_matrix_part(&self.elements[0], &rhs.elements, 3),
            ],
            [
                Self::multiply_matrix_part(&self.elements[1], &rhs.elements, 0),
                Self::multiply_matrix_part(&self.elements[1], &rhs.elements, 1),
                Self::multiply_matrix_part(&self.elements[1], &rhs.elements, 2),
                Self::multiply_matrix_part(&self.elements[1], &rhs.elements, 3),
            ],
            [
                Self::multiply_matrix_part(&self.elements[2], &rhs.elements, 0),
                Self::multiply_matrix_part(&self.elements[2], &rhs.elements, 1),
                Self::multiply_matrix_part(&self.elements[2], &rhs.elements, 2),
                Self::multiply_matrix_part(&self.elements[2], &rhs.elements, 3),
            ],
            [
                Self::multiply_matrix_part(&self.elements[3], &rhs.elements, 0),
                Self::multiply_matrix_part(&self.elements[3], &rhs.elements, 1),
                Self::multiply_matrix_part(&self.elements[3], &rhs.elements, 2),
                Self::multiply_matrix_part(&self.elements[3], &rhs.elements, 3),
            ],
        ])
    }
}
