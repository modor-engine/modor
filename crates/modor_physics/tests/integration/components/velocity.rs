use approx::assert_abs_diff_eq;
use modor_math::Vector3D;
use modor_physics::{RelativeVelocity, Velocity};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_2d_velocity() {
    let velocity = Velocity::xy(1., 2.);
    assert_abs_diff_eq!(velocity.x, 1.);
    assert_abs_diff_eq!(velocity.y, 2.);
    assert_abs_diff_eq!(velocity.z, 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_3d_velocity() {
    let velocity = Velocity::xyz(1., 2., 3.);
    assert_abs_diff_eq!(velocity.x, 1.);
    assert_abs_diff_eq!(velocity.y, 2.);
    assert_abs_diff_eq!(velocity.z, 3.);
    assert_abs_diff_eq!(velocity.components().0, 1.);
    assert_abs_diff_eq!(velocity.components().1, 2.);
    assert_abs_diff_eq!(velocity.components().2, 3.);
    let velocity = Velocity::create(4., 5., 6.);
    assert_abs_diff_eq!(velocity.x, 4.);
    assert_abs_diff_eq!(velocity.y, 5.);
    assert_abs_diff_eq!(velocity.z, 6.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_2d_relative_velocity() {
    let velocity = RelativeVelocity::xy(1., 2.);
    assert_abs_diff_eq!(velocity.x, 1.);
    assert_abs_diff_eq!(velocity.y, 2.);
    assert_abs_diff_eq!(velocity.z, 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_3d_relative_velocity() {
    let velocity = RelativeVelocity::xyz(1., 2., 3.);
    assert_abs_diff_eq!(velocity.x, 1.);
    assert_abs_diff_eq!(velocity.y, 2.);
    assert_abs_diff_eq!(velocity.z, 3.);
    assert_abs_diff_eq!(velocity.components().0, 1.);
    assert_abs_diff_eq!(velocity.components().1, 2.);
    assert_abs_diff_eq!(velocity.components().2, 3.);
    let velocity = RelativeVelocity::create(4., 5., 6.);
    assert_abs_diff_eq!(velocity.x, 4.);
    assert_abs_diff_eq!(velocity.y, 5.);
    assert_abs_diff_eq!(velocity.z, 6.);
}
