use crate::Vec3;
use std::ops::Mul;

/// A 4x4 matrix.
#[derive(Clone, Debug)]
pub struct Mat4 {
    elements: [[f32; 4]; 4],
}

impl Mat4 {
    /// Creates a new matrix from `elements` in an array of arrays.
    ///
    /// Each array of `elements` corresponds to a line of the matrix.
    #[inline]
    pub fn from_array(elements: [[f32; 4]; 4]) -> Self {
        Self { elements }
    }

    /// Creates a new transform matrix from a `position` and a `size`.
    pub fn from_position_scale(position: Vec3, scale: Vec3) -> Self {
        Self::from_array([
            [scale.x, 0., 0., 0.],
            [0., scale.y, 0., 0.],
            [0., 0., scale.z, 0.],
            [position.x, position.y, position.z, 1.],
        ])
    }

    /// Returns the array of arrays containing the elements of the matrix.
    ///
    /// Each array of the array corresponds to a line of the matrix.
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

impl Mul<Vec3> for Mat4 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        let point = [rhs.x, rhs.y, rhs.z, 1.];
        Vec3::xyz(
            Self::multiply_matrix_part(&point, &self.elements, 0),
            Self::multiply_matrix_part(&point, &self.elements, 1),
            Self::multiply_matrix_part(&point, &self.elements, 2),
        )
    }
}

impl Mul<Self> for Mat4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::from_array([
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
