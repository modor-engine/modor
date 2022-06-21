use approx::assert_abs_diff_eq;
use modor_math::{Mat4, Quat, Vec2, Vec3};
use std::f32::consts::FRAC_PI_2;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_from_array() {
    let mat = Mat4::from_array([
        [1., 2., 3., 4.],
        [5., 6., 7., 8.],
        [9., 10., 11., 12.],
        [13., 14., 15., 16.],
    ]);
    assert_abs_diff_eq!(mat.to_array()[0][0], 1.);
    assert_abs_diff_eq!(mat.to_array()[0][1], 2.);
    assert_abs_diff_eq!(mat.to_array()[0][2], 3.);
    assert_abs_diff_eq!(mat.to_array()[0][3], 4.);
    assert_abs_diff_eq!(mat.to_array()[1][0], 5.);
    assert_abs_diff_eq!(mat.to_array()[1][1], 6.);
    assert_abs_diff_eq!(mat.to_array()[1][2], 7.);
    assert_abs_diff_eq!(mat.to_array()[1][3], 8.);
    assert_abs_diff_eq!(mat.to_array()[2][0], 9.);
    assert_abs_diff_eq!(mat.to_array()[2][1], 10.);
    assert_abs_diff_eq!(mat.to_array()[2][2], 11.);
    assert_abs_diff_eq!(mat.to_array()[2][3], 12.);
    assert_abs_diff_eq!(mat.to_array()[3][0], 13.);
    assert_abs_diff_eq!(mat.to_array()[3][1], 14.);
    assert_abs_diff_eq!(mat.to_array()[3][2], 15.);
    assert_abs_diff_eq!(mat.to_array()[3][3], 16.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_from_position_scale() {
    let mat = Mat4::from_position_scale(Vec3::xyz(1., 2., 3.), Vec3::xyz(4., 5., 6.));
    assert_abs_diff_eq!(mat.to_array()[0][0], 4.);
    assert_abs_diff_eq!(mat.to_array()[0][1], 0.);
    assert_abs_diff_eq!(mat.to_array()[0][2], 0.);
    assert_abs_diff_eq!(mat.to_array()[0][3], 0.);
    assert_abs_diff_eq!(mat.to_array()[1][0], 0.);
    assert_abs_diff_eq!(mat.to_array()[1][1], 5.);
    assert_abs_diff_eq!(mat.to_array()[1][2], 0.);
    assert_abs_diff_eq!(mat.to_array()[1][3], 0.);
    assert_abs_diff_eq!(mat.to_array()[2][0], 0.);
    assert_abs_diff_eq!(mat.to_array()[2][1], 0.);
    assert_abs_diff_eq!(mat.to_array()[2][2], 6.);
    assert_abs_diff_eq!(mat.to_array()[2][3], 0.);
    assert_abs_diff_eq!(mat.to_array()[3][0], 1.);
    assert_abs_diff_eq!(mat.to_array()[3][1], 2.);
    assert_abs_diff_eq!(mat.to_array()[3][2], 3.);
    assert_abs_diff_eq!(mat.to_array()[3][3], 1.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_from_position() {
    let mat = Mat4::from_position(Vec3::xyz(1., 2., 3.));
    assert_abs_diff_eq!(mat.to_array()[0][0], 1.);
    assert_abs_diff_eq!(mat.to_array()[0][1], 0.);
    assert_abs_diff_eq!(mat.to_array()[0][2], 0.);
    assert_abs_diff_eq!(mat.to_array()[0][3], 0.);
    assert_abs_diff_eq!(mat.to_array()[1][0], 0.);
    assert_abs_diff_eq!(mat.to_array()[1][1], 1.);
    assert_abs_diff_eq!(mat.to_array()[1][2], 0.);
    assert_abs_diff_eq!(mat.to_array()[1][3], 0.);
    assert_abs_diff_eq!(mat.to_array()[2][0], 0.);
    assert_abs_diff_eq!(mat.to_array()[2][1], 0.);
    assert_abs_diff_eq!(mat.to_array()[2][2], 1.);
    assert_abs_diff_eq!(mat.to_array()[2][3], 0.);
    assert_abs_diff_eq!(mat.to_array()[3][0], 1.);
    assert_abs_diff_eq!(mat.to_array()[3][1], 2.);
    assert_abs_diff_eq!(mat.to_array()[3][2], 3.);
    assert_abs_diff_eq!(mat.to_array()[3][3], 1.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_from_scale() {
    let mat = Mat4::from_scale(Vec3::xyz(4., 5., 6.));
    assert_abs_diff_eq!(mat.to_array()[0][0], 4.);
    assert_abs_diff_eq!(mat.to_array()[0][1], 0.);
    assert_abs_diff_eq!(mat.to_array()[0][2], 0.);
    assert_abs_diff_eq!(mat.to_array()[0][3], 0.);
    assert_abs_diff_eq!(mat.to_array()[1][0], 0.);
    assert_abs_diff_eq!(mat.to_array()[1][1], 5.);
    assert_abs_diff_eq!(mat.to_array()[1][2], 0.);
    assert_abs_diff_eq!(mat.to_array()[1][3], 0.);
    assert_abs_diff_eq!(mat.to_array()[2][0], 0.);
    assert_abs_diff_eq!(mat.to_array()[2][1], 0.);
    assert_abs_diff_eq!(mat.to_array()[2][2], 6.);
    assert_abs_diff_eq!(mat.to_array()[2][3], 0.);
    assert_abs_diff_eq!(mat.to_array()[3][0], 0.);
    assert_abs_diff_eq!(mat.to_array()[3][1], 0.);
    assert_abs_diff_eq!(mat.to_array()[3][2], 0.);
    assert_abs_diff_eq!(mat.to_array()[3][3], 1.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn mul_vec2() {
    let rotation = Quat::from_z(FRAC_PI_2).matrix();
    let vec = rotation * Vec2::xy(1., 1.);
    assert_abs_diff_eq!(vec.x, 1.);
    assert_abs_diff_eq!(vec.y, -1.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn mul_vec3() {
    let rotation = Quat::from_x(FRAC_PI_2).matrix();
    let vec = rotation * Vec3::xyz(1., 1., 1.);
    assert_abs_diff_eq!(vec.x, 1.);
    assert_abs_diff_eq!(vec.y, 1.);
    assert_abs_diff_eq!(vec.z, -1.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn mul_mat() {
    let translation_scale =
        Mat4::from_position_scale(Vec3::xyz(2., 4., 6.), Vec3::xyz(0.1, 0.2, 0.3));
    let rotation = Quat::from_x(FRAC_PI_2).matrix();
    let vec = rotation * translation_scale * Vec3::xyz(1., 1., 1.);
    assert_abs_diff_eq!(vec.x, 2.1);
    assert_abs_diff_eq!(vec.y, 4.2);
    assert_abs_diff_eq!(vec.z, 5.7);
    let rotation = Quat::from_y(FRAC_PI_2).matrix();
    let vec = rotation * translation_scale * Vec3::xyz(1., 1., 1.);
    assert_abs_diff_eq!(vec.x, 1.9);
    assert_abs_diff_eq!(vec.y, 4.2);
    assert_abs_diff_eq!(vec.z, 6.3);
    let rotation = Quat::from_z(FRAC_PI_2).matrix();
    let vec = rotation * translation_scale * Vec3::xyz(1., 1., 1.);
    assert_abs_diff_eq!(vec.x, 2.1);
    assert_abs_diff_eq!(vec.y, 3.8);
    assert_abs_diff_eq!(vec.z, 6.3);
}
