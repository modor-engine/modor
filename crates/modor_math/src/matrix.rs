use crate::Point3D;

pub fn multiply_matrices(left: [[f32; 4]; 4], right: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    [
        [
            multiply_matrix_part(&left[0], &right, 0),
            multiply_matrix_part(&left[0], &right, 1),
            multiply_matrix_part(&left[0], &right, 2),
            multiply_matrix_part(&left[0], &right, 3),
        ],
        [
            multiply_matrix_part(&left[1], &right, 0),
            multiply_matrix_part(&left[1], &right, 1),
            multiply_matrix_part(&left[1], &right, 2),
            multiply_matrix_part(&left[1], &right, 3),
        ],
        [
            multiply_matrix_part(&left[2], &right, 0),
            multiply_matrix_part(&left[2], &right, 1),
            multiply_matrix_part(&left[2], &right, 2),
            multiply_matrix_part(&left[2], &right, 3),
        ],
        [
            multiply_matrix_part(&left[3], &right, 0),
            multiply_matrix_part(&left[3], &right, 1),
            multiply_matrix_part(&left[3], &right, 2),
            multiply_matrix_part(&left[3], &right, 3),
        ],
    ]
}

pub fn multiply_matrix_and_point_3d<P>(point: P, matrix: [[f32; 4]; 4]) -> P
where
    P: Point3D,
{
    let (x, y, z) = point.components();
    let vec = [x, y, z, 1.];
    P::create(
        multiply_matrix_part(&vec, &matrix, 0),
        multiply_matrix_part(&vec, &matrix, 1),
        multiply_matrix_part(&vec, &matrix, 2),
    )
}

fn multiply_matrix_part(part: &[f32; 4], other_matrix: &[[f32; 4]; 4], j: usize) -> f32 {
    (0..4)
        .map(|k| part[k] * other_matrix[k][j])
        .reduce(|a, b| a + b)
        .expect("internal error: wrong matrix size")
}
