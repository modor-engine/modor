use approx::assert_abs_diff_eq;
use modor_math::Vector3D;
use modor_physics::{Acceleration, RelativeAcceleration};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_2d_acceleration() {
    let acceleration = Acceleration::xy(1., 2.);
    assert_abs_diff_eq!(acceleration.x, 1.);
    assert_abs_diff_eq!(acceleration.y, 2.);
    assert_abs_diff_eq!(acceleration.z, 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_3d_acceleration() {
    let acceleration = Acceleration::xyz(1., 2., 3.);
    assert_abs_diff_eq!(acceleration.x, 1.);
    assert_abs_diff_eq!(acceleration.y, 2.);
    assert_abs_diff_eq!(acceleration.z, 3.);
    assert_abs_diff_eq!(acceleration.components().0, 1.);
    assert_abs_diff_eq!(acceleration.components().1, 2.);
    assert_abs_diff_eq!(acceleration.components().2, 3.);
    let acceleration = Acceleration::create(4., 5., 6.);
    assert_abs_diff_eq!(acceleration.x, 4.);
    assert_abs_diff_eq!(acceleration.y, 5.);
    assert_abs_diff_eq!(acceleration.z, 6.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_2d_relative_acceleration() {
    let acceleration = RelativeAcceleration::xy(1., 2.);
    assert_abs_diff_eq!(acceleration.x, 1.);
    assert_abs_diff_eq!(acceleration.y, 2.);
    assert_abs_diff_eq!(acceleration.z, 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_3d_relative_acceleration() {
    let acceleration = RelativeAcceleration::xyz(1., 2., 3.);
    assert_abs_diff_eq!(acceleration.x, 1.);
    assert_abs_diff_eq!(acceleration.y, 2.);
    assert_abs_diff_eq!(acceleration.z, 3.);
    assert_abs_diff_eq!(acceleration.components().0, 1.);
    assert_abs_diff_eq!(acceleration.components().1, 2.);
    assert_abs_diff_eq!(acceleration.components().2, 3.);
    let acceleration = RelativeAcceleration::create(4., 5., 6.);
    assert_abs_diff_eq!(acceleration.x, 4.);
    assert_abs_diff_eq!(acceleration.y, 5.);
    assert_abs_diff_eq!(acceleration.z, 6.);
}
